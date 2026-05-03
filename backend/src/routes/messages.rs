use std::convert::Infallible;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::sse::{Event, KeepAlive, Sse},
};
use futures_util::{Stream, StreamExt};
use serde_json::json;
use sqlx::{Row, Sqlite, SqlitePool, Transaction};
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

const MAX_STREAMED_ASSISTANT_BYTES: usize = 128 * 1024;
const MAX_HISTORY_MESSAGES: i64 = 200;
const MAX_HISTORY_BYTES: usize = 128 * 1024;

pub async fn list(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<Message>>, AppError> {
    ensure_conversation_exists(&state.pool, id).await?;

    let rows = sqlx::query(
        "SELECT id, conversation_id, role, content, created_at
         FROM messages
         WHERE conversation_id = ?
         ORDER BY created_at ASC, CASE role WHEN 'user' THEN 0 ELSE 1 END, id ASC",
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

    let mut history = load_history(&state.pool, conv_id).await?;
    history.push(ChatMessage {
        role: Role::User.as_str().to_string(),
        content: body.content.clone(),
    });

    let assistant_text = state
        .openrouter
        .chat(&state.config.openrouter_model, history)
        .await?;

    let user_id = Uuid::new_v4();
    let user_now = now_iso();
    let assistant_id = Uuid::new_v4();
    let assistant_now = now_iso();

    let mut tx = state.pool.begin().await?;
    insert_message_tx(
        &mut tx,
        user_id,
        conv_id,
        Role::User,
        &body.content,
        &user_now,
    )
    .await?;
    insert_message_tx(
        &mut tx,
        assistant_id,
        conv_id,
        Role::Assistant,
        &assistant_text,
        &assistant_now,
    )
    .await?;
    touch_conversation_tx(&mut tx, conv_id, &assistant_now).await?;
    tx.commit().await?;

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

    let mut history = load_history(&state.pool, conv_id).await?;
    history.push(ChatMessage {
        role: Role::User.as_str().to_string(),
        content: body.content.clone(),
    });
    let mut upstream = state
        .openrouter
        .stream(&state.config.openrouter_model, history)
        .await?;

    let user_id = Uuid::new_v4();
    let user_now = now_iso();
    let user_content = body.content;

    let (tx, rx) = mpsc::channel::<Result<Event, Infallible>>(32);

    let pool = state.pool.clone();
    tokio::spawn(async move {
        let mut accumulated = String::new();
        let mut errored = false;
        let mut client_gone = false;

        while let Some(item) = upstream.next().await {
            match item {
                Ok(token) => {
                    if accumulated.len() + token.len() > MAX_STREAMED_ASSISTANT_BYTES {
                        errored = true;
                        let payload = json!({
                            "code": "UPSTREAM",
                            "message": "Provider unavailable",
                        })
                        .to_string();
                        let _ = tx
                            .send(Ok(Event::default().event("error").data(payload)))
                            .await;
                        break;
                    }
                    accumulated.push_str(&token);
                    let event = Event::default().event("token").data(token);
                    if tx.send(Ok(event)).await.is_err() {
                        client_gone = true;
                        break;
                    }
                }
                Err(e) => {
                    errored = true;
                    tracing::warn!(error = ?e, "upstream stream failed");
                    let payload = json!({
                        "code": "UPSTREAM",
                        "message": e.client_message(),
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
        if errored {
            // Intentional: upstream failures and over-cap streams discard both
            // messages so the user can retry without committing a broken turn.
            return;
        }

        let mut tx_db = match pool.begin().await {
            Ok(tx) => tx,
            Err(e) => {
                tracing::error!(error = ?e, "begin stream persistence transaction failed");
                send_stream_error(&tx, "Provider unavailable").await;
                return;
            }
        };
        if let Err(e) = insert_message_tx(
            &mut tx_db,
            user_id,
            conv_id,
            Role::User,
            &user_content,
            &user_now,
        )
        .await
        {
            tracing::error!(error = ?e, "persist user msg failed");
            send_stream_error(&tx, "Provider unavailable").await;
            return;
        }
        if !accumulated.is_empty()
            && let Err(e) = insert_message_tx(
                &mut tx_db,
                assistant_id,
                conv_id,
                Role::Assistant,
                &accumulated,
                &assistant_now,
            )
            .await
        {
            tracing::error!(error = ?e, "persist assistant msg failed");
            send_stream_error(&tx, "Provider unavailable").await;
            return;
        }
        let touch_time = if accumulated.is_empty() {
            &user_now
        } else {
            &assistant_now
        };
        if let Err(e) = touch_conversation_tx(&mut tx_db, conv_id, touch_time).await {
            tracing::error!(error = ?e, "touch conversation failed");
            send_stream_error(&tx, "Provider unavailable").await;
            return;
        }
        if let Err(e) = tx_db.commit().await {
            tracing::error!(error = ?e, "commit stream persistence failed");
            send_stream_error(&tx, "Provider unavailable").await;
            return;
        }

        if client_gone {
            return;
        }

        if accumulated.is_empty() {
            send_stream_error(&tx, "Empty response").await;
            return;
        }

        let payload = json!({
            "message_id": assistant_id.to_string(),
        })
        .to_string();
        let event = Event::default().event("done").data(payload);
        let _ = tx.send(Ok(event)).await;
    });

    let stream = ReceiverStream::new(rx);
    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

async fn send_stream_error(tx: &mpsc::Sender<Result<Event, Infallible>>, message: &str) {
    let payload = json!({
        "code": "UPSTREAM",
        "message": message,
    })
    .to_string();
    let event = Event::default().event("error").data(payload);
    let _ = tx.send(Ok(event)).await;
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

async fn insert_message_tx(
    tx: &mut Transaction<'_, Sqlite>,
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
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn load_history(pool: &SqlitePool, conv_id: Uuid) -> Result<Vec<ChatMessage>, AppError> {
    let rows = sqlx::query(
        "SELECT role, content FROM (
            SELECT role, content, created_at, id
            FROM messages
            WHERE conversation_id = ?
            ORDER BY created_at DESC, id DESC
            LIMIT ?
        ) ORDER BY created_at ASC, CASE role WHEN 'user' THEN 0 ELSE 1 END, id ASC",
    )
    .bind(conv_id.to_string())
    .bind(MAX_HISTORY_MESSAGES)
    .fetch_all(pool)
    .await?;

    let mut history = Vec::with_capacity(rows.len());
    let mut total_bytes = 0usize;
    for row in rows.into_iter().rev() {
        let role: String = row.try_get("role")?;
        let content: String = row.try_get("content")?;
        let content_len = content.len();
        if !history.is_empty() && total_bytes + content_len > MAX_HISTORY_BYTES {
            break;
        }
        total_bytes += content_len;
        history.push(ChatMessage { role, content });
    }
    history.reverse();
    Ok(history)
}

async fn touch_conversation_tx(
    tx: &mut Transaction<'_, Sqlite>,
    conv_id: Uuid,
    now: &str,
) -> Result<(), AppError> {
    sqlx::query("UPDATE conversations SET updated_at = ? WHERE id = ?")
        .bind(now)
        .bind(conv_id.to_string())
        .execute(&mut **tx)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;
    use tempfile::TempDir;

    async fn temp_pool() -> (SqlitePool, &'static TempDir) {
        let dir = TempDir::new().expect("tempdir");
        let dir_ref: &'static TempDir = Box::leak(Box::new(dir));
        let db_path = dir_ref.path().join("history.sqlite");
        let url = format!(
            "sqlite://{}?mode=rwc",
            db_path.display().to_string().replace('\\', "/")
        );
        (db::init_pool(&url).await.expect("init pool"), dir_ref)
    }

    #[tokio::test]
    async fn load_history_keeps_newest_messages_within_byte_cap() {
        let (pool, _dir) = temp_pool().await;
        let conv_id = Uuid::new_v4();
        let created_at = "2026-05-03T00:00:00Z";
        sqlx::query(
            "INSERT INTO conversations (id, title, created_at, updated_at) VALUES (?, ?, ?, ?)",
        )
        .bind(conv_id.to_string())
        .bind("history cap")
        .bind(created_at)
        .bind(created_at)
        .execute(&pool)
        .await
        .unwrap();

        let mut tx = pool.begin().await.unwrap();
        for idx in 0..4 {
            let content = format!("{idx}:{}", "x".repeat(40_000));
            let created_at = format!("2026-05-03T00:00:0{idx}Z");
            insert_message_tx(
                &mut tx,
                Uuid::new_v4(),
                conv_id,
                Role::User,
                &content,
                &created_at,
            )
            .await
            .unwrap();
        }
        tx.commit().await.unwrap();

        let history = load_history(&pool, conv_id).await.unwrap();

        assert_eq!(history.len(), 3);
        assert!(history.iter().all(|msg| msg.content.len() <= 40_002));
        assert!(history[0].content.starts_with("1:"));
        assert!(history[1].content.starts_with("2:"));
        assert!(history[2].content.starts_with("3:"));
        assert!(history.iter().map(|msg| msg.content.len()).sum::<usize>() <= MAX_HISTORY_BYTES);
    }
}
