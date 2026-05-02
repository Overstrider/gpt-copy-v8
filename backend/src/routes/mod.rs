pub mod conversations;
pub mod health;
pub mod messages;

use axum::{
    Router,
    http::{HeaderValue, Method, header},
    routing::{get, post},
};
use sqlx::SqlitePool;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use uuid::Uuid;

use crate::{error::AppError, state::AppState};

pub fn router(state: AppState) -> Router {
    let origin: HeaderValue = state
        .config
        .frontend_origin
        .parse()
        .expect("FRONTEND_ORIGIN must be a valid origin");

    let cors = CorsLayer::new()
        .allow_origin(origin)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE]);

    Router::new()
        .route("/health", get(health::get_health))
        .route(
            "/api/conversations",
            get(conversations::list).post(conversations::create),
        )
        .route(
            "/api/conversations/:id/messages",
            get(messages::list).post(messages::send),
        )
        .route("/api/conversations/:id/stream", post(messages::stream))
        .with_state(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}

pub(crate) async fn ensure_conversation_exists(
    pool: &SqlitePool,
    id: Uuid,
) -> Result<(), AppError> {
    let row = sqlx::query("SELECT id FROM conversations WHERE id = ?")
        .bind(id.to_string())
        .fetch_optional(pool)
        .await?;
    if row.is_none() {
        return Err(AppError::NotFound(format!("conversation {id} not found")));
    }
    Ok(())
}
