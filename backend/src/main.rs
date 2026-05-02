use std::{net::SocketAddr, sync::Arc};

use anyhow::Context;
use gpt_copy_v8_backend::{
    build_app,
    config::Config,
    db,
    openrouter::{HttpOpenRouterClient, OpenRouterClient},
    state::AppState,
};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Best-effort: load root .env then local .env.
    let _ = dotenvy::from_path("../.env");
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cfg = Config::from_env().context("load config")?;
    let pool = db::init_pool(&cfg.database_url)
        .await
        .context("init db pool")?;

    let openrouter: Arc<dyn OpenRouterClient> = Arc::new(
        HttpOpenRouterClient::new(cfg.openrouter_api_key.clone())
            .context("build openrouter client")?,
    );

    let state = AppState {
        pool,
        openrouter,
        config: Arc::new(cfg.clone()),
    };

    let app = build_app(state);

    let addr: SocketAddr = format!("{}:{}", cfg.backend_host, cfg.backend_port)
        .parse()
        .context("parse backend addr")?;
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("bind {addr}"))?;
    tracing::info!(%addr, "backend listening");
    axum::serve(listener, app).await.context("serve")?;
    Ok(())
}
