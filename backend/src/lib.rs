pub mod config;
pub mod db;
pub mod error;
pub mod models;
pub mod openrouter;
pub mod routes;
pub mod state;

pub fn build_app(state: state::AppState) -> axum::Router {
    routes::router(state)
}
