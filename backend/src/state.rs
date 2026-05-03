use std::sync::Arc;

use sqlx::SqlitePool;

use crate::{config::Config, openrouter::OpenRouterClient};

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub openrouter: Arc<dyn OpenRouterClient>,
    pub config: Arc<Config>,
}
