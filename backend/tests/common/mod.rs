#![allow(dead_code)]

use std::sync::Arc;

use axum::{Router, body::Body, response::Response};
use gpt_copy_v8_backend::{
    build_app,
    config::Config,
    db,
    openrouter::{MockOpenRouterClient, OpenRouterClient},
    state::AppState,
};
use http_body_util::BodyExt;
use serde_json::Value;
use tempfile::TempDir;

pub async fn test_app() -> Router {
    test_app_with_mock(MockOpenRouterClient::with_chat("ok")).await
}

pub async fn test_app_with_mock(mock: MockOpenRouterClient) -> Router {
    let dir = TempDir::new().expect("tempdir");
    // Leak the dir to keep the SQLite file alive for the lifetime of the test.
    let dir_ref: &'static TempDir = Box::leak(Box::new(dir));
    let db_path = dir_ref.path().join("test.sqlite");
    let url = format!(
        "sqlite://{}?mode=rwc",
        db_path.display().to_string().replace('\\', "/")
    );
    let pool = db::init_pool(&url).await.expect("init pool");

    let cfg = Config {
        openrouter_api_key: "test".to_string(),
        openrouter_model: "test-model".to_string(),
        database_url: url,
        backend_host: "127.0.0.1".to_string(),
        backend_port: 0,
        frontend_origin: "http://localhost:3000".to_string(),
    };

    let openrouter: Arc<dyn OpenRouterClient> = Arc::new(mock);
    let state = AppState {
        pool,
        openrouter,
        config: Arc::new(cfg),
    };
    build_app(state)
}

pub async fn read_json(res: Response) -> Value {
    let bytes = res
        .into_body()
        .collect()
        .await
        .expect("collect body")
        .to_bytes();
    serde_json::from_slice(&bytes).expect("parse json")
}

pub async fn read_text(res: Response) -> String {
    let bytes = res
        .into_body()
        .collect()
        .await
        .expect("collect body")
        .to_bytes();
    String::from_utf8(bytes.to_vec()).expect("utf8")
}

pub fn json_body(v: Value) -> Body {
    Body::from(serde_json::to_vec(&v).expect("serialize"))
}
