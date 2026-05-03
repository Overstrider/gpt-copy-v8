use axum::{Json, extract::State, http::StatusCode};
use sqlx::Row;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::{Conversation, CreateConversationReq, now_iso, validate_title},
    state::AppState,
};

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<Conversation>>, AppError> {
    let rows = sqlx::query(
        "SELECT id, title, created_at, updated_at FROM conversations ORDER BY updated_at DESC, id DESC",
    )
    .fetch_all(&state.pool)
    .await?;

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let id_str: String = row.try_get("id")?;
        let id = Uuid::parse_str(&id_str)
            .map_err(|_| AppError::Internal("invalid uuid in db".to_string()))?;
        out.push(Conversation {
            id,
            title: row.try_get("title")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        });
    }

    Ok(Json(out))
}

pub async fn create(
    State(state): State<AppState>,
    Json(body): Json<CreateConversationReq>,
) -> Result<(StatusCode, Json<Conversation>), AppError> {
    let title = body.title.trim().to_string();
    validate_title(&title)?;

    let id = Uuid::new_v4();
    let now = now_iso();

    sqlx::query(
        "INSERT INTO conversations (id, title, created_at, updated_at) VALUES (?, ?, ?, ?)",
    )
    .bind(id.to_string())
    .bind(&title)
    .bind(&now)
    .bind(&now)
    .execute(&state.pool)
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(Conversation {
            id,
            title,
            created_at: now.clone(),
            updated_at: now,
        }),
    ))
}
