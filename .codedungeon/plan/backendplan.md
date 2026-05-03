# backendplan

## meta
- repo: backend
- lang: Rust 2024
- stack: axum 0.7 + sqlx 0.8 (sqlite) + tokio 1 + reqwest 0.12 + tracing 0.1
- project_mode: BOOTSTRAP
- PROJECT_RULES_STATUS: approved
- PROJECT_RULES_DIGEST: 54eae19e90a1b403912baee95e0294f1ff7bdf2aff6afb8d07af8d0744677dfe
- PROJECT_RULES_READ: yes

## scope
Bootstrap Rust 2024 Axum API at `backend/`. SQLite persistence via sqlx runtime queries. Server-side OpenRouter proxy (sync + SSE). REST endpoints for conversations + messages. Strict env-driven config. Trait-based OpenRouter client → mockable tests. Integration tests via `build_app(state)` + `tower::ServiceExt::oneshot`. Verified by `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`.

## dependencies
- root `.env` w/ `OPENROUTER_API_KEY` set → backend boots (`Config::from_env` fails fast w/o key).
- migration `0001_init.sql` runs before any handler touches DB.
- `FRONTEND_ORIGIN` matches frontend dev origin → CORS allows req.

## files

### backend/Cargo.toml
- type: new-module
- location: `backend/Cargo.toml`
- description: crate manifest. edition 2024. `[lib]` + `[[bin]] name="server" path="src/main.rs"`. Crate name `gpt_copy_v8_backend`.
- notes: bin uses `src/main.rs`; lib exposes `build_app` + modules for tests.

#### rust
- package: `name = "gpt_copy_v8_backend"`, `version = "0.1.0"`, `edition = "2024"`.
- `[lib] path = "src/lib.rs"`.
- `[[bin]] name = "server" path = "src/main.rs"`.
- `[dependencies]` exact pins:
  - `axum = { version = "0.7", features = ["macros"] }`
  - `tokio = { version = "1", features = ["full"] }`
  - `tower-http = { version = "0.5", features = ["cors", "trace"] }`
  - `sqlx = { version = "0.8", default-features = false, features = ["runtime-tokio-rustls", "sqlite", "uuid", "chrono", "macros", "migrate"] }`
  - `serde = { version = "1", features = ["derive"] }`
  - `serde_json = "1"`
  - `reqwest = { version = "0.12", default-features = false, features = ["json", "stream", "rustls-tls"] }`
  - `futures-util = "0.3"`
  - `tokio-stream = "0.1"`
  - `uuid = { version = "1", features = ["v4", "serde"] }`
  - `chrono = { version = "0.4", features = ["serde"] }`
  - `tracing = "0.1"`
  - `tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }`
  - `thiserror = "1"`
  - `dotenvy = "0.15"`
  - `async-trait = "0.1"`
  - `anyhow = "1"`  (top-level glue inside `main.rs` only)
- `[dev-dependencies]`:
  - `mockito = "1"`
  - `tower = { version = "0.5", features = ["util"] }`
  - `tempfile = "3"`
  - `http-body-util = "0.1"`  (drain `Response<Body>` in oneshot tests)
- error-mapping: omit (manifest only).

### backend/rust-toolchain.toml
- type: new-module
- location: `backend/rust-toolchain.toml`
- description: pin stable channel; ensure 2024 edition support.
- notes: components limited to default profile.

#### rust
- TOML body:
  - `[toolchain]`
  - `channel = "stable"`
  - `components = ["rustfmt", "clippy"]`
  - `profile = "minimal"`

### backend/migrations/0001_init.sql
- type: migration
- location: `backend/migrations/0001_init.sql`
- description: initial schema. `conversations` + `messages` w/ FK cascade. Index on (conversation_id, created_at).
- database: two tables + 1 index.
- notes: TEXT-typed timestamps (ISO 8601 strings) — matches model serde strings, simpler than chrono mapping.

#### rust
- SQL exactly as arcplan §requirements.2:
  - `CREATE TABLE conversations (id TEXT PRIMARY KEY, title TEXT NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL);`
  - `CREATE TABLE messages (id TEXT PRIMARY KEY, conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE, role TEXT NOT NULL CHECK(role IN ('user','assistant','system')), content TEXT NOT NULL, created_at TEXT NOT NULL);`
  - `CREATE INDEX idx_messages_conversation ON messages(conversation_id, created_at);`
- runtime: applied via `sqlx::migrate!("./migrations").run(&pool)` in `db::init_pool`.

### backend/src/lib.rs
- type: new-module
- location: `backend/src/lib.rs`
- description: library root. Re-exports modules + exposes `build_app(state) -> Router` for tests.
- references: arcplan §affected-modules `lib.rs`.

#### rust
- module decls: `pub mod config; pub mod db; pub mod error; pub mod models; pub mod openrouter; pub mod routes; pub mod state;`
- fn signature: `pub fn build_app(state: state::AppState) -> axum::Router` — delegates to `routes::router(state)`.
- no other logic. Tests + bin both consume `build_app`.

### backend/src/main.rs
- type: new-module
- location: `backend/src/main.rs`
- description: entry. Loads env, inits tracing, opens DB pool, runs migrations, builds OpenRouter HTTP client, builds router, binds `host:port`, serves.
- references: arcplan §affected-modules `main.rs`.

#### rust
- attribute: `#[tokio::main]`
- signature: `async fn main() -> anyhow::Result<()>`
- order:
  1. `dotenvy::from_path("../.env").ok();` then `dotenvy::dotenv().ok();` (best effort).
  2. `tracing_subscriber::fmt().with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"))).init();`
  3. `let cfg = Config::from_env()?;`
  4. `let pool = db::init_pool(&cfg.database_url).await?;`
  5. `let openrouter = Arc::new(openrouter::HttpOpenRouterClient::new(cfg.openrouter_api_key.clone())?) as Arc<dyn OpenRouterClient + Send + Sync>;`
  6. `let state = AppState { pool, openrouter, config: Arc::new(cfg.clone()) };`
  7. `let app = build_app(state);`
  8. `let addr = format!("{}:{}", cfg.backend_host, cfg.backend_port).parse::<SocketAddr>()?;`
  9. `let listener = tokio::net::TcpListener::bind(addr).await?;`
  10. `tracing::info!(%addr, "backend listening");`
  11. `axum::serve(listener, app).await?;`
  12. `Ok(())`
- error-mapping: bubble via `?` → exit non-zero on failure. `anyhow` permitted here only.

### backend/src/config.rs
- type: new-module
- location: `backend/src/config.rs`
- description: env parsing. Single source of truth. Fails fast on missing required keys.
- references: arcplan §requirements.3.

#### rust
- struct:
  ```rust
  #[derive(Debug, Clone)]
  pub struct Config {
      pub openrouter_api_key: String,
      pub openrouter_model: String,
      pub database_url: String,
      pub backend_host: String,
      pub backend_port: u16,
      pub frontend_origin: String,
  }
  ```
- error enum (module-local):
  ```rust
  #[derive(Debug, thiserror::Error)]
  pub enum ConfigError {
      #[error("missing required env var: {0}")]
      Missing(&'static str),
      #[error("invalid env var {0}: {1}")]
      Invalid(&'static str, String),
  }
  ```
- impl: `pub fn from_env() -> Result<Self, ConfigError>`.
- defaults:
  - `openrouter_model` → `"nvidia/nemotron-3-super-120b-a12b:free"`
  - `database_url` → `"sqlite://backend/gpt-copy-v8.sqlite?mode=rwc"`
  - `backend_host` → `"127.0.0.1"`
  - `backend_port` → `8080`
  - `frontend_origin` → `"http://localhost:3000"`
- required: `OPENROUTER_API_KEY` only — empty/missing → `ConfigError::Missing("OPENROUTER_API_KEY")`.
- port parse: `u16::from_str_radix(&val, 10).map_err(|e| ConfigError::Invalid("BACKEND_PORT", e.to_string()))`.

### backend/src/error.rs
- type: new-module
- location: `backend/src/error.rs`
- description: app-wide error enum + `IntoResponse` mapping → JSON body.
- references: arcplan §requirements.4.

#### rust
- enum:
  ```rust
  #[derive(Debug, thiserror::Error)]
  pub enum AppError {
      #[error("validation: {0}")]
      Validation(String),
      #[error("not found: {0}")]
      NotFound(String),
      #[error("database error")]
      Database(#[from] sqlx::Error),
      #[error("upstream: {0}")]
      Upstream(String),
      #[error("internal: {0}")]
      Internal(String),
  }
  ```
- response body type:
  ```rust
  #[derive(serde::Serialize)]
  struct ErrorBody<'a> { error: ErrorInner<'a> }
  #[derive(serde::Serialize)]
  struct ErrorInner<'a> { code: &'a str, message: String }
  ```
- impl `IntoResponse for AppError`:
  - match self → `(status, code)`:
    - `Validation(msg)` → `(StatusCode::BAD_REQUEST, "VALIDATION", msg)`
    - `NotFound(msg)` → `(StatusCode::NOT_FOUND, "NOT_FOUND", msg)`
    - `Upstream(msg)` → `(StatusCode::BAD_GATEWAY, "UPSTREAM", msg)`
    - `Database(e)` → log via `tracing::error!(?e)`, then `(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL", "database error".into())`
    - `Internal(msg)` → log, `(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL", msg)`
  - return `(status, Json(ErrorBody { error: ErrorInner { code, message } })).into_response()`.
- `From<sqlx::Error> for AppError` → derived via `#[from]` on `Database` variant.
- additional: `impl From<openrouter::OpenRouterError> for AppError` → maps `OpenRouterError::HttpStatus(429|503|5xx)` → `AppError::Upstream`, `OpenRouterError::Decode|Stream` → `AppError::Upstream`, `OpenRouterError::Transport(reqwest)` → `AppError::Upstream`.

### backend/src/state.rs
- type: new-module
- location: `backend/src/state.rs`
- description: shared `AppState` cloned per req. Holds DB pool, OpenRouter client trait obj, config Arc.
- references: arcplan §requirements (state cloned via axum `State<AppState>`).

#### rust
- struct:
  ```rust
  #[derive(Clone)]
  pub struct AppState {
      pub pool: sqlx::SqlitePool,
      pub openrouter: std::sync::Arc<dyn crate::openrouter::OpenRouterClient + Send + Sync>,
      pub config: std::sync::Arc<crate::config::Config>,
  }
  ```
- no methods. Constructed in `main.rs` + `tests/common.rs`.

### backend/src/db.rs
- type: new-module
- location: `backend/src/db.rs`
- description: pool init + migration runner. WAL + busy_timeout pragmas on connect.
- references: arcplan §risks "SQLite write contention".

#### rust
- fn signature:
  ```rust
  pub async fn init_pool(database_url: &str) -> Result<sqlx::SqlitePool, sqlx::Error>
  ```
- impl:
  ```rust
  use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
  use std::str::FromStr;
  let opts = SqliteConnectOptions::from_str(database_url)?
      .create_if_missing(true)
      .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
      .busy_timeout(std::time::Duration::from_secs(5));
  let pool = SqlitePoolOptions::new()
      .max_connections(5)
      .connect_with(opts).await?;
  sqlx::migrate!("./migrations").run(&pool).await?;
  Ok(pool)
  ```
- migration runner uses `sqlx::migrate!` (compile-time embedded — only macro permitted; arcplan §11 forbids `query!` macros, not `migrate!`).

### backend/src/models.rs
- type: new-module
- location: `backend/src/models.rs`
- description: DB rows + DTOs. ISO-8601 timestamp strings (TEXT in SQLite).
- references: arcplan §requirements.2 (schema).

#### rust
- enum:
  ```rust
  #[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
  #[serde(rename_all = "lowercase")]
  pub enum Role {
      User,
      Assistant,
      System,
  }
  impl Role {
      pub fn as_str(&self) -> &'static str { match self { Self::User => "user", Self::Assistant => "assistant", Self::System => "system" } }
      pub fn parse(s: &str) -> Option<Self> { match s { "user" => Some(Self::User), "assistant" => Some(Self::Assistant), "system" => Some(Self::System), _ => None } }
  }
  ```
- struct:
  ```rust
  #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
  pub struct Conversation {
      pub id: uuid::Uuid,
      pub title: String,
      pub created_at: String,
      pub updated_at: String,
  }
  ```
- struct:
  ```rust
  #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
  pub struct Message {
      pub id: uuid::Uuid,
      pub conversation_id: uuid::Uuid,
      pub role: Role,
      pub content: String,
      pub created_at: String,
  }
  ```
- request DTOs:
  ```rust
  #[derive(Debug, serde::Deserialize)]
  pub struct CreateConversationReq { pub title: String }
  #[derive(Debug, serde::Deserialize)]
  pub struct CreateMessageReq { pub content: String }
  ```
- response DTO:
  ```rust
  #[derive(Debug, serde::Serialize)]
  pub struct SendMessageRes {
      pub user_message: Message,
      pub assistant_message: Message,
  }
  ```
- helper: `pub fn now_iso() -> String { chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true) }`.
- validation helpers:
  - `pub fn validate_title(t: &str) -> Result<(), crate::error::AppError>` → trim empty / >200 → `AppError::Validation`.
  - `pub fn validate_content(c: &str) -> Result<(), crate::error::AppError>` → empty / >32_000 → `AppError::Validation`.

### backend/src/openrouter.rs
- type: new-service
- location: `backend/src/openrouter.rs`
- description: trait + HTTP impl + Mock impl. Sync + streaming chat completions via OpenRouter.
- references: arcplan §requirements.5; risks 429/timeout.

#### rust
- error enum:
  ```rust
  #[derive(Debug, thiserror::Error)]
  pub enum OpenRouterError {
      #[error("transport: {0}")]
      Transport(#[from] reqwest::Error),
      #[error("upstream status: {0}")]
      HttpStatus(u16),
      #[error("decode: {0}")]
      Decode(String),
      #[error("stream: {0}")]
      Stream(String),
  }
  ```
- request structs:
  ```rust
  #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
  pub struct ChatMessage {
      pub role: String,    // "user" | "assistant" | "system"
      pub content: String,
  }
  #[derive(Debug, serde::Serialize)]
  struct ChatCompletionReq<'a> {
      model: &'a str,
      messages: &'a [ChatMessage],
      stream: bool,
  }
  #[derive(Debug, serde::Deserialize)]
  struct ChatCompletionRes {
      choices: Vec<ChoiceFull>,
  }
  #[derive(Debug, serde::Deserialize)]
  struct ChoiceFull { message: AssistantMsg }
  #[derive(Debug, serde::Deserialize)]
  struct AssistantMsg { content: String }
  #[derive(Debug, serde::Deserialize)]
  struct StreamChunk { choices: Vec<ChoiceDelta> }
  #[derive(Debug, serde::Deserialize)]
  struct ChoiceDelta { delta: Delta }
  #[derive(Debug, serde::Deserialize)]
  struct Delta { #[serde(default)] content: Option<String> }
  ```
- trait:
  ```rust
  use futures_util::stream::BoxStream;
  #[async_trait::async_trait]
  pub trait OpenRouterClient: Send + Sync {
      async fn chat(&self, model: &str, messages: Vec<ChatMessage>) -> Result<String, OpenRouterError>;
      async fn stream(&self, model: &str, messages: Vec<ChatMessage>) -> Result<BoxStream<'static, Result<String, OpenRouterError>>, OpenRouterError>;
  }
  ```
- HTTP impl:
  ```rust
  pub struct HttpOpenRouterClient {
      http: reqwest::Client,
      api_key: String,
      base_url: String,
  }
  impl HttpOpenRouterClient {
      pub fn new(api_key: String) -> Result<Self, OpenRouterError> { /* build client w/ 30s timeout */ }
      pub fn with_base_url(api_key: String, base_url: String) -> Result<Self, OpenRouterError> { /* test override */ }
  }
  ```
- chat impl:
  - POST `{base_url}/chat/completions` (default `https://openrouter.ai/api/v1`).
  - headers: `Authorization: Bearer <api_key>`, `HTTP-Referer: http://localhost`, `X-Title: gpt-copy-v8`.
  - body: `ChatCompletionReq { model, messages, stream: false }`.
  - status check: `!status.is_success()` → `Err(HttpStatus(status.as_u16()))`.
  - decode → `ChatCompletionRes` → `choices[0].message.content` (else `Decode("missing choices".into())`).
- stream impl:
  - same POST, `stream: true`.
  - response `bytes_stream()` → wrap in custom adapter that buffers bytes, splits on `\n\n`, parses lines starting with `data: `.
  - line == `data: [DONE]` → terminate stream cleanly.
  - line `data: <json>` → `serde_json::from_str::<StreamChunk>(json)` → emit `delta.content` (skip if `None`).
  - decode error → `Stream(...)` Err item; transport error → propagate.
  - return `BoxStream<'static, Result<String, OpenRouterError>>`.
- Mock impl (test-only, gated `#[cfg(any(test, feature = "test-mock"))]` or simply public for dev tests):
  ```rust
  pub struct MockOpenRouterClient {
      pub chat_response: std::sync::Mutex<Result<String, OpenRouterError>>,
      pub stream_chunks: std::sync::Mutex<Vec<Result<String, OpenRouterError>>>,
  }
  impl MockOpenRouterClient {
      pub fn with_chat(text: impl Into<String>) -> Self { /* ... */ }
      pub fn with_stream(chunks: Vec<&str>) -> Self { /* ... */ }
      pub fn with_error(err: OpenRouterError) -> Self { /* ... */ }
  }
  #[async_trait::async_trait]
  impl OpenRouterClient for MockOpenRouterClient { /* return canned data */ }
  ```
- error-mapping: `From<OpenRouterError> for AppError` lives in `error.rs` (see above) — all variants → `AppError::Upstream(msg)`.

### backend/src/routes/mod.rs
- type: new-module
- location: `backend/src/routes/mod.rs`
- description: aggregate child routers. Attach CORS + trace layers. Single `pub fn router(state) -> Router`.
- references: arcplan §requirements.6, .7.

#### rust
- mod decls: `pub mod conversations; pub mod health; pub mod messages;`
- fn signature: `pub fn router(state: crate::state::AppState) -> axum::Router`
- impl:
  ```rust
  use axum::{Router, routing::{get, post}};
  use tower_http::{cors::{Any, CorsLayer}, trace::TraceLayer};
  use http::{Method, header};

  let cors = CorsLayer::new()
      .allow_origin(state.config.frontend_origin.parse::<axum::http::HeaderValue>().expect("FRONTEND_ORIGIN must be valid origin"))
      .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
      .allow_headers([header::CONTENT_TYPE]);

  Router::new()
      .route("/health", get(health::get_health))
      .route("/api/conversations", get(conversations::list).post(conversations::create))
      .route("/api/conversations/:id/messages", get(messages::list).post(messages::send))
      .route("/api/conversations/:id/stream", post(messages::stream))
      .with_state(state)
      .layer(cors)
      .layer(TraceLayer::new_for_http())
  ```

### backend/src/routes/health.rs
- type: new-endpoint
- location: `backend/src/routes/health.rs`
- description: liveness probe.
- input: none.
- output: `{ "status": "ok", "version": "<crate version>" }` 200.

#### rust
- handler:
  ```rust
  pub async fn get_health() -> axum::Json<serde_json::Value> {
      axum::Json(serde_json::json!({
          "status": "ok",
          "version": env!("CARGO_PKG_VERSION"),
      }))
  }
  ```
- error-mapping: none (infallible).

### backend/src/routes/conversations.rs
- type: new-endpoint
- location: `backend/src/routes/conversations.rs`
- description: list + create conversations.
- references: arcplan §contracts GET/POST `/api/conversations`.

#### rust
- list handler:
  ```rust
  pub async fn list(
      State(state): State<AppState>,
  ) -> Result<Json<Vec<Conversation>>, AppError>
  ```
  - query: `sqlx::query("SELECT id, title, created_at, updated_at FROM conversations ORDER BY updated_at DESC")`.
  - `.fetch_all(&state.pool).await?`.
  - map row → `Conversation { id: Uuid::parse_str(row.get::<&str,_>("id"))?, title: row.get("title"), created_at: row.get("created_at"), updated_at: row.get("updated_at") }`.
  - parse error → `AppError::Internal("invalid uuid in db".into())`.
  - return `Json(vec)`, 200.
- create handler:
  ```rust
  pub async fn create(
      State(state): State<AppState>,
      Json(body): Json<CreateConversationReq>,
  ) -> Result<(StatusCode, Json<Conversation>), AppError>
  ```
  - call `validate_title(&body.title)?`.
  - generate `id = Uuid::new_v4()`, `now = now_iso()`.
  - `sqlx::query("INSERT INTO conversations (id, title, created_at, updated_at) VALUES (?, ?, ?, ?)").bind(id.to_string()).bind(&body.title).bind(&now).bind(&now).execute(&state.pool).await?`.
  - return `(StatusCode::CREATED, Json(Conversation { id, title: body.title, created_at: now.clone(), updated_at: now }))`.
- error-mapping: `Validation` → 400 (via `AppError::IntoResponse`); DB → 500.

### backend/src/routes/messages.rs
- type: new-endpoint
- location: `backend/src/routes/messages.rs`
- description: list messages, send (sync), stream (SSE).
- references: arcplan §contracts; §requirements.5, .8, .9.

#### rust
- list handler:
  ```rust
  pub async fn list(
      State(state): State<AppState>,
      Path(id): Path<Uuid>,
  ) -> Result<Json<Vec<Message>>, AppError>
  ```
  - call `ensure_conversation_exists(&state.pool, id).await?` (helper below).
  - `sqlx::query("SELECT id, conversation_id, role, content, created_at FROM messages WHERE conversation_id = ? ORDER BY created_at ASC").bind(id.to_string()).fetch_all(&state.pool).await?`.
  - map row → `Message`. Role parse via `Role::parse(row.get("role")).ok_or_else(|| AppError::Internal("bad role".into()))?`.
- send handler (sync):
  ```rust
  pub async fn send(
      State(state): State<AppState>,
      Path(conv_id): Path<Uuid>,
      Json(body): Json<CreateMessageReq>,
  ) -> Result<(StatusCode, Json<SendMessageRes>), AppError>
  ```
  - `validate_content(&body.content)?`.
  - `ensure_conversation_exists(&state.pool, conv_id).await?`.
  - load existing msgs → build `Vec<ChatMessage>`.
  - `tx = state.pool.begin().await?` for atomic insert of user + assistant after upstream succeeds; revert if upstream fails by NOT committing user msg until after success.
    - Actually: persist user msg first (commit), call OpenRouter, persist assistant msg. Failure between → user msg remains (visible in history), assistant absent. Mirror real chat UX. Document choice here.
  - INSERT user msg row.
  - history: query `messages` for conv → map to `Vec<ChatMessage>` (include just-inserted user msg).
  - `let assistant_text = state.openrouter.chat(&state.config.openrouter_model, history).await.map_err(AppError::from)?;`
  - INSERT assistant msg row.
  - update `conversations.updated_at`.
  - return `(StatusCode::CREATED, Json(SendMessageRes { user_message, assistant_message }))`.
- stream handler (SSE):
  ```rust
  use axum::response::sse::{Event, KeepAlive, Sse};
  use futures_util::Stream;
  pub async fn stream(
      State(state): State<AppState>,
      Path(conv_id): Path<Uuid>,
      Json(body): Json<CreateMessageReq>,
  ) -> Result<Sse<impl Stream<Item = Result<Event, std::convert::Infallible>>>, AppError>
  ```
  - same prelude: validate, ensure conv exists, build history.
  - call `state.openrouter.stream(&model, history).await?` → upstream stream of token deltas.
  - inside async block via `async_stream::stream!` or hand-built unfolding:
    - for each token Ok(t): `accumulated.push_str(&t)`; yield `Ok(Event::default().event("token").data(t))`.
    - on Err(e): yield `Ok(Event::default().event("error").data(json!({ "code": "UPSTREAM", "message": e.to_string() }).to_string()))`; break.
    - on stream end: persist user msg + assistant msg w/ accumulated content, update `conversations.updated_at`, yield `Ok(Event::default().event("done").data(json!({ "message_id": assistant_id }).to_string()))`.
  - on client disconnect: detect `mpsc::Sender` close and return before DB persistence. Do not persist partial assistant content or the paired user message; this keeps retry semantics deterministic.
  - `Sse::new(stream).keep_alive(KeepAlive::default())`.
- helper (private to `messages.rs` or `routes/mod.rs`):
  ```rust
  async fn ensure_conversation_exists(pool: &SqlitePool, id: Uuid) -> Result<(), AppError> {
      let row = sqlx::query("SELECT id FROM conversations WHERE id = ?")
          .bind(id.to_string())
          .fetch_optional(pool)
          .await?;
      row.ok_or_else(|| AppError::NotFound(format!("conversation {id} not found"))).map(|_| ())
  }
  ```
  - Note: dedicated `ensure_conversation_exists` placed in `routes/mod.rs` as `pub(super) async fn` so both list and send + stream reuse it.
- async: streaming relay = single tokio task per req. No `spawn_blocking` needed (all async I/O). `mpsc::channel(32)` for backpressure.
- error-mapping: validation → 400; conv missing → 404; upstream → 502; DB → 500.

### backend/tests/health.rs
- type: integration-test
- location: `backend/tests/health.rs`
- description: `GET /health` → 200 + body shape.
- references: arcplan §requirements.10.

#### rust
- helper module `mod common;` (file `tests/common/mod.rs` — see below).
- test:
  ```rust
  #[tokio::test]
  async fn health_returns_ok() {
      let app = common::test_app().await;
      let res = app.oneshot(Request::get("/health").body(Body::empty()).unwrap()).await.unwrap();
      assert_eq!(res.status(), StatusCode::OK);
      let body = read_json(res).await;
      assert_eq!(body["status"], "ok");
      assert!(body["version"].is_string());
  }
  ```

### backend/tests/conversations.rs
- type: integration-test
- location: `backend/tests/conversations.rs`
- description: validation + persistence.

#### rust
- cases:
  1. `create_rejects_empty_title` → POST `{ "title": "" }` → 400, body `error.code == "VALIDATION"`.
  2. `create_rejects_too_long_title` → 201-char title → 400.
  3. `create_persists_and_list_returns` → POST valid → 201, then GET `/api/conversations` → array contains created id, title matches.
  4. `create_returns_uuid_v4` → assert `Uuid::parse_str(body["id"]).is_ok()`.

### backend/tests/messages.rs
- type: integration-test
- location: `backend/tests/messages.rs`
- description: send (sync) + stream w/ Mock client.

#### rust
- cases:
  1. `send_rejects_empty_content` → 400.
  2. `send_rejects_too_long_content` → 32_001 chars → 400.
  3. `send_returns_user_and_assistant` → mock returns `"Hello back"` → POST returns 201, body has both messages, assistant role + content match.
  4. `send_404_when_conv_missing` → unknown UUID → 404.
  5. `send_502_when_upstream_fails` → mock w/ `OpenRouterError::HttpStatus(429)` → 502, code `UPSTREAM`.
  6. `list_messages_returns_in_order` → seed 3 messages → GET returns sorted by created_at asc.
  7. `stream_emits_tokens_then_done` → mock stream yields `["Hel", "lo"]` → consume SSE → assert events `token: "Hel"`, `token: "lo"`, `done` w/ message_id.
- SSE parse helper: read response body bytes, split on `\n\n`, find lines `event:` + `data:`.

### backend/tests/openrouter_mock.rs
- type: integration-test
- location: `backend/tests/openrouter_mock.rs`
- description: trait impl behavior + error paths via `mockito` for `HttpOpenRouterClient`.

#### rust
- cases:
  1. `http_chat_success` → mockito returns `{"choices":[{"message":{"content":"hi"}}]}` → `client.chat(...)` returns `"hi"`.
  2. `http_chat_429_maps_to_http_status` → mockito returns 429 → `Err(OpenRouterError::HttpStatus(429))`.
  3. `http_stream_parses_data_lines` → mockito returns event-stream body w/ 2 chunks + `[DONE]` → collected stream yields `["Hel","lo"]` then ends.
  4. `http_stream_skips_empty_delta` → chunk w/ no content field → no item emitted.
  5. `http_stream_terminates_on_done` → consumer iter ends after `[DONE]` regardless of trailing bytes.
- use `HttpOpenRouterClient::with_base_url` to point at mockito server URL.

### backend/tests/common/mod.rs
- type: test-helper (not in arcplan affected-modules list, but required by arcplan §requirements.10 "share helper `fn test_app()`").
- location: `backend/tests/common/mod.rs`
- description: shared `test_app()` builder + utilities. Each test file declares `mod common;`.

#### rust
- fn signatures:
  ```rust
  pub async fn test_app() -> axum::Router { test_app_with_mock(MockOpenRouterClient::with_chat("ok")).await }
  pub async fn test_app_with_mock(mock: MockOpenRouterClient) -> axum::Router { /* tempfile DB + run migrations + AppState */ }
  pub async fn read_json(res: axum::response::Response) -> serde_json::Value { /* http_body_util::BodyExt::collect + serde_json::from_slice */ }
  pub fn json_body(v: serde_json::Value) -> axum::body::Body { axum::body::Body::from(serde_json::to_vec(&v).unwrap()) }
  ```
- DB setup:
  - `let dir = tempfile::tempdir().unwrap();`
  - `let db_path = dir.path().join("test.sqlite");`
  - `let url = format!("sqlite://{}?mode=rwc", db_path.display());`
  - `let pool = db::init_pool(&url).await.unwrap();`
  - keep `dir` alive: leak via `Box::leak` OR store inside a `static OnceCell<TempDir>` per test using a unique tempdir per `test_app` call (returned `Router` wraps state holding pool → dir freed after pool drop; safer: leak the `TempDir` for test simplicity).
- config: `Config { openrouter_api_key: "test".into(), openrouter_model: "test-model".into(), database_url: url, backend_host: "127.0.0.1".into(), backend_port: 0, frontend_origin: "http://localhost:3000".into() }`.

## test-strategy

- runtime: `#[tokio::test]` for all integration tests (need axum + sqlx async).
- transport: `tower::ServiceExt::oneshot` to invoke handlers without binding sockets (arcplan §convention).
- DB: tempfile-backed SQLite DB per `test_app()` call. Migrations via `db::init_pool`. Isolated per test → no cross-test contamination.
- OpenRouter: never call real API. `MockOpenRouterClient` for handler tests; `mockito` for `HttpOpenRouterClient` itself.
- assertion shape:
  - status code via `res.status()`.
  - body parse via shared `read_json` helper → `serde_json::Value`.
  - DB state post-call via direct `sqlx::query("SELECT ...").fetch_all(&pool)` (only when needed; mostly assert through API list endpoints).
- SSE tests: collect full body bytes, parse `event:`/`data:` lines, assert event kinds + payloads in order.
- coverage matrix mapped to arcplan §requirements:
  - .3 (config) → covered indirectly via `test_app_with_mock` building Config.
  - .4 (errors) → `conversations.rs` validation + missing-conv tests confirm body shape.
  - .5 (OpenRouter) → `openrouter_mock.rs` covers HTTP + stream parsing.
  - .8 (SSE) → `messages.rs::stream_emits_tokens_then_done`.
  - .9 (validation limits) → conversations + messages length tests.

## verification

- `cd backend && cargo fmt --check`  → formatting must pass.
- `cd backend && cargo clippy --all-targets -- -D warnings`  → no clippy warnings, treat as errors.
- `cd backend && cargo test`  → all integration tests pass.
- `cd backend && cargo build --release`  → release builds (smoke).
- run order: fmt → clippy → test. fmt + clippy must pass before test execution counts as PASS.

## execution-order
1. `Cargo.toml` + `rust-toolchain.toml` → toolchain pinned, deps resolvable.
2. `migrations/0001_init.sql` → schema source of truth.
3. `src/lib.rs` skeleton + `src/config.rs` + `src/error.rs` + `src/models.rs` → core types compile.
4. `src/db.rs` → pool + migrate fn.
5. `src/openrouter.rs` (trait + Mock + HTTP impl) → handlers can wire.
6. `src/state.rs` → glue.
7. `src/routes/{health,conversations,messages,mod}.rs` → handlers.
8. `src/main.rs` → bootstrap.
9. `tests/common/mod.rs` → shared helper.
10. `tests/{health,conversations,messages,openrouter_mock}.rs` → integration coverage.
reason: types + DB before handlers; client before handlers; helpers before tests.

PLAN_COMPLETE: backendplan.md
