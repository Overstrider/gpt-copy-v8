use std::convert::Infallible;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::sse::{Event, KeepAlive, Sse},
};
use futures_util::{Stream, StreamExt};
use serde_json::json;
use sqlx::{Row, SqlitePool};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::{CreateMessageReq, Message, Role, SendMessageRes, now_iso, validate_content},
    openrouter::ChatMessage,
    routes::ensure_conversation_exists,
    state::AppState,
};

pub async fn list(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<Message>>, AppError> {
    ensure_conversation_exists(&state.pool, id).await?;

    let rows = sqlx::query(
        "SELECT id, conversation_id, role, content, created_at FROM messages WHERE conversation_id = ? ORDER BY created_at ASC, id ASC",
    )
    .bind(id.to_string())
    .fetch_all(&state.pool)
    .await?;

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        out.push(row_to_message(&row)?);
    }
    Ok(Json(out))
}

pub async fn send(
    State(state): State<AppState>,
    Path(conv_id): Path<Uuid>,
    Json(body): Json<CreateMessageReq>,
) -> Result<(StatusCode, Json<SendMessageRes>), AppError> {
    validate_content(&body.content)?;
    ensure_conversation_exists(&state.pool, conv_id).await?;

    // Persist user message first.
    let user_id = Uuid::new_v4();
    let user_now = now_iso();
    insert_message(
        &state.pool,
        user_id,
        conv_id,
        Role::User,
        &body.content,
        &user_now,
    )
    .await?;

    // Build history including the new user msg.
    let history = load_history(&state.pool, conv_id).await?;

    // Call OpenRouter (sync). On failure, the user msg remains.
    let assistant_text = state
        .openrouter
        .chat(&state.config.openrouter_model, history)
        .await?;

    let assistant_id = Uuid::new_v4();
    let assistant_now = now_iso();
    insert_message(
        &state.pool,
        assistant_id,
        conv_id,
        Role::Assistant,
        &assistant_text,
        &assistant_now,
    )
    .await?;

    touch_conversation(&state.pool, conv_id, &assistant_now).await?;

    let user_message = Message {
        id: user_id,
        conversation_id: conv_id,
        role: Role::User,
        content: body.content,
        created_at: user_now,
    };
    let assistant_message = Message {
        id: assistant_id,
        conversation_id: conv_id,
        role: Role::Assistant,
        content: assistant_text,
        created_at: assistant_now,
    };

    Ok((
        StatusCode::CREATED,
        Json(SendMessageRes {
            user_message,
            assistant_message,
        }),
    ))
}

pub async fn stream(
    State(state): State<AppState>,
    Path(conv_id): Path<Uuid>,
    Json(body): Json<CreateMessageReq>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, AppError> {
    validate_content(&body.content)?;
    ensure_conversation_exists(&state.pool, conv_id).await?;

    let user_id = Uuid::new_v4();
    let user_now = now_iso();
    insert_message(
        &state.pool,
        user_id,
        conv_id,
        Role::User,
        &body.content,
        &user_now,
    )
    .await?;

    let history = load_history(&state.pool, conv_id).await?;
    let mut upstream = state
        .openrouter
        .stream(&state.config.openrouter_model, history)
        .await?;

    let (tx, rx) = mpsc::channel::<Result<Event, Infallible>>(32);

    let pool = state.pool.clone();
    tokio::spawn(async move {
        let mut accumulated = String::new();
        let mut errored = false;

        while let Some(item) = upstream.next().await {
            match item {
                Ok(token) => {
                    accumulated.push_str(&token);
                    let event = Event::default().event("token").data(token);
                    if tx.send(Ok(event)).await.is_err() {
                        // Client disconnected — persist partial.
                        break;
                    }
                }
                Err(e) => {
                    errored = true;
                    let payload = json!({
                        "code": "UPSTREAM",
                        "message": e.to_string(),
                    })
                    .to_string();
                    let event = Event::default().event("error").data(payload);
                    let _ = tx.send(Ok(event)).await;
                    break;
                }
            }
        }

        let assistant_id = Uuid::new_v4();
        let assistant_now = now_iso();
        if !accumulated.is_empty() {
            if let Err(e) = insert_message(
                &pool,
                assistant_id,
                conv_id,
                Role::Assistant,
                &accumulated,
                &assistant_now,
            )
            .await
            {
                tracing::error!(error = ?e, "persist assistant msg failed");
            }
            let _ = touch_conversation(&pool, conv_id, &assistant_now).await;
        }

        if !errored {
            let payload = json!({
                "message_id": assistant_id.to_string(),
            })
            .to_string();
            let event = Event::default().event("done").data(payload);
            let _ = tx.send(Ok(event)).await;
        }
    });

    let stream = ReceiverStream::new(rx);
    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

fn row_to_message(row: &sqlx::sqlite::SqliteRow) -> Result<Message, AppError> {
    let id_str: String = row.try_get("id")?;
    let conv_str: String = row.try_get("conversation_id")?;
    let role_str: String = row.try_get("role")?;
    let id = Uuid::parse_str(&id_str)
        .map_err(|_| AppError::Internal("invalid msg uuid in db".to_string()))?;
    let conversation_id = Uuid::parse_str(&conv_str)
        .map_err(|_| AppError::Internal("invalid conv uuid in db".to_string()))?;
    let role = Role::parse(&role_str).ok_or_else(|| AppError::Internal("bad role".to_string()))?;
    Ok(Message {
        id,
        conversation_id,
        role,
        content: row.try_get("content")?,
        created_at: row.try_get("created_at")?,
    })
}

async fn insert_message(
    pool: &SqlitePool,
    id: Uuid,
    conv_id: Uuid,
    role: Role,
    content: &str,
    created_at: &str,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO messages (id, conversation_id, role, content, created_at) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(id.to_string())
    .bind(conv_id.to_string())
    .bind(role.as_str())
    .bind(content)
    .bind(created_at)
    .execute(pool)
    .await?;
    Ok(())
}

async fn load_history(pool: &SqlitePool, conv_id: Uuid) -> Result<Vec<ChatMessage>, AppError> {
    let rows = sqlx::query(
        "SELECT role, content FROM messages WHERE conversation_id = ? ORDER BY created_at ASC, id ASC",
    )
    .bind(conv_id.to_string())
    .fetch_all(pool)
    .await?;

    let mut history = Vec::with_capacity(rows.len());
    for row in rows {
        let role: String = row.try_get("role")?;
        let content: String = row.try_get("content")?;
        history.push(ChatMessage { role, content });
    }
    Ok(history)
}

async fn touch_conversation(pool: &SqlitePool, conv_id: Uuid, now: &str) -> Result<(), AppError> {
    sqlx::query("UPDATE conversations SET updated_at = ? WHERE id = ?")
        .bind(now)
        .bind(conv_id.to_string())
        .execute(pool)
        .await?;
    Ok(())
}
