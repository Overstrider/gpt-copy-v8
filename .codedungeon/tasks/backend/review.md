# Backend Review — combined TASK-002, 004, 005, 007, 011

PROJECT_RULES_STATUS: approved
PROJECT_RULES_DIGEST: 54eae19e90a1b403912baee95e0294f1ff7bdf2aff6afb8d07af8d0744677dfe
PROJECT_RULES_READ: yes

Verdict: APPROVED

## Verification

cargo fmt --check → clean.
cargo clippy --all-targets -- -D warnings → clean.
cargo test → 20 pass (1 health, 5 conv, 8 msg, 6 openrouter).
cargo build --release → OK.

## TASK-002 Foundation + Persistence

backend/Cargo.toml → edition "2024", crate gpt_copy_v8_backend, [lib] + [[bin]] name="server" → matches arcplan §requirements.1.
backend/rust-toolchain.toml → channel stable + rustfmt + clippy.
backend/migrations/0001_init.sql → conversations + messages w/ FK CASCADE + role CHECK + idx_messages_conversation → matches arcplan §requirements.2.
backend/src/config.rs → Config::from_env fails fast on missing OPENROUTER_API_KEY; defaults match rules (model nvidia/nemotron-3-super-120b-a12b:free, host 127.0.0.1, port 8080, origin http://localhost:3000).
backend/src/error.rs → AppError variants → 400/404/500/502/500 + body { error: { code, message } } → matches §requirements.4.
backend/src/db.rs → SqliteConnectOptions WAL + busy_timeout 5s + max_connections 5 + sqlx::migrate! → matches risks "SQLite write contention".
backend/src/state.rs → AppState clones pool + Arc<dyn OpenRouterClient> + Arc<Config>.
backend/src/routes/health.rs → GET /health → 200 { status: "ok", version: env!(CARGO_PKG_VERSION) } → matches §requirements (health).
backend/tests/health.rs → asserts 200 + body shape via tower::oneshot → matches §requirements.10.

## TASK-004 OpenRouter Client

backend/src/openrouter.rs → trait OpenRouterClient with chat + stream → matches §requirements.5.
HttpOpenRouterClient → POST {base}/chat/completions, bearer auth, HTTP-Referer + X-Title headers → matches §requirements.5.
Stream parser → splits buffer on \n\n, skips lines without "data:", terminates on [DONE], yields delta.content via extract_delta helper, skips empty content → matches §requirements.5.
Bug fixed mid-loop: [DONE] now drains s.pending before returning None → tokens before [DONE] not lost.
Error mapping → OpenRouterError variants Transport/HttpStatus/Decode/Stream → AppError::Upstream via From impl in error.rs → matches §requirements.4.
MockOpenRouterClient → with_chat / with_stream / with_chat_error / with_stream_error → deterministic + injectable failures.
backend/tests/openrouter_mock.rs → 6 cases: success, 429 → HttpStatus, decode err on no choices, stream parse two tokens, skip empty delta, terminate on [DONE].
No real upstream calls in tests — mockito only.
No Authorization header logged.

## TASK-005 Conversation Routes

backend/src/routes/conversations.rs → list (GET /api/conversations) ORDER BY updated_at DESC, id DESC → tiebreaker for deterministic ordering when timestamps collide.
create (POST) → validates title, generates Uuid::new_v4, inserts, returns 201 → matches §contracts.
backend/src/models.rs → validate_title rejects empty (after trim) + >200 chars → AppError::Validation.
backend/tests/conversations.rs → 5 tests: empty list, reject empty, reject 201-char, persists+lists, uuid v4. All pass.

## TASK-007 Message Send + SSE

backend/src/routes/messages.rs → list (sorted ASC), send (sync), stream (SSE) → matches §contracts.
list → ensure_conversation_exists → 404 if missing → matches.
send → validate_content → ensure_conv → insert user → load_history → openrouter.chat → insert assistant → touch_conversation updated_at → 201 SendMessageRes.
stream → spawn tokio task w/ mpsc::channel(32) → relay tokens → on stream end persist user + accumulated assistant msg and emit done with message_id → matches §requirements.8.
On upstream err → emit event "error" + payload { code: "UPSTREAM", message } → no panic.
On client disconnect (tx send err) → break loop → discard in-flight user/assistant turn so retry can start from clean state.
backend/tests/messages.rs → 8 tests: validation 400, 404, 502 upstream, persists+lists, stream tokens+done. All pass.

## TASK-011 Verification Gate

cargo fmt --check → PASS.
cargo clippy --all-targets -- -D warnings → PASS.
cargo test → 20 pass / 0 fail.
cargo build --release → PASS in 24s.
No external OpenRouter calls in test suite (mockito for HTTP, MockOpenRouterClient for handler tests).

## Notes

CORS layer in routes/mod.rs uses parse on FRONTEND_ORIGIN → expect panic if invalid; acceptable for boot fail-fast since Config::from_env loads default known-valid origin.
runtime sqlx::query! macros NOT used → only sqlx::query (runtime) → matches §requirements.11 + Prime Directive.
sqlx::migrate! macro IS used → permitted (compile-time embedded migrations not banned by §11).
.env not committed (verified in repo state — only backend/ files added).

REVIEW_COMPLETE: backend
