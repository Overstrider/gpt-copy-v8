pub mod conversations;
pub mod health;
pub mod messages;

use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{
    Json, Router,
    body::Body,
    extract::State,
    http::{HeaderValue, Method, Request, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde_json::json;
use sqlx::SqlitePool;
use tokio::sync::Mutex;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use uuid::Uuid;

use crate::{error::AppError, state::AppState};

pub fn router(state: AppState) -> Router {
    let origin: HeaderValue = state
        .config
        .frontend_origin
        .parse()
        .unwrap_or_else(|e| {
            tracing::error!(error = ?e, origin = %state.config.frontend_origin, "invalid FRONTEND_ORIGIN; falling back to localhost");
            HeaderValue::from_static("http://localhost:3000")
        });

    let cors = CorsLayer::new()
        .allow_origin(origin)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE]);
    let limiter = Arc::new(RateLimiter::new(60, Duration::from_secs(60)));

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
        .layer(middleware::from_fn_with_state(limiter, rate_limit))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}

struct RateLimiter {
    max: u32,
    window: Duration,
    buckets: Mutex<HashMap<String, RateBucket>>,
}

struct RateBucket {
    start: Instant,
    count: u32,
}

impl RateLimiter {
    fn new(max: u32, window: Duration) -> Self {
        Self {
            max,
            window,
            buckets: Mutex::new(HashMap::new()),
        }
    }

    async fn allow(&self, key: String) -> bool {
        let now = Instant::now();
        let mut buckets = self.buckets.lock().await;
        let bucket = buckets.entry(key).or_insert(RateBucket {
            start: now,
            count: 0,
        });
        if now.duration_since(bucket.start) >= self.window {
            bucket.start = now;
            bucket.count = 0;
        }
        if bucket.count >= self.max {
            return false;
        }
        bucket.count += 1;
        true
    }
}

async fn rate_limit(
    State(limiter): State<Arc<RateLimiter>>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let key = rate_limit_key(&req);
    if limiter.allow(key).await {
        return next.run(req).await;
    }
    (
        StatusCode::TOO_MANY_REQUESTS,
        Json(json!({
            "error": {
                "code": "RATE_LIMITED",
                "message": "too many requests"
            }
        })),
    )
        .into_response()
}

fn rate_limit_key(req: &Request<Body>) -> String {
    req.headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .unwrap_or("local")
        .to_string()
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
