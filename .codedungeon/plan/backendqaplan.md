# qaplan: backend

## meta
- repo: backend
- lang: Rust 2024
- stack: axum 0.7 + sqlx 0.8 (sqlite) + reqwest 0.12 + tokio 1
- project_mode: BOOTSTRAP
- PROJECT_RULES_STATUS: approved
- PROJECT_RULES_DIGEST: 54eae19e90a1b403912baee95e0294f1ff7bdf2aff6afb8d07af8d0744677dfe
- PROJECT_RULES_READ: yes

## test-strategy
- lang: Rust 2024
- test-types: integration, api
- integration-framework: cargo test (detected from `backend/Cargo.toml` edition 2024 + `[dev-dependencies] mockito tower tempfile http-body-util`)
- api-test-approach: live HTTP via curl against `cargo run --bin server` on `127.0.0.1:8080`. mock OpenRouter unavailable for live curl → API tests gated on either real `OPENROUTER_API_KEY` OR a build-flag/env that swaps in `MockOpenRouterClient` in `main.rs`. Plan recommends env `BACKEND_USE_MOCK_OPENROUTER=1` for API-test runs.
- e2e-framework: none (backend repo)
- existing-tests-dir: none (BOOTSTRAP — `backend/tests/` created by impl tasks)
- existing-patterns: none (first feature). Patterns prescribed by `backendplan.md` §files `tests/*.rs` + `tests/common/mod.rs`.
- test-runner-command: `cargo test --manifest-path backend/Cargo.toml`
- auth: none (no auth layer in v0). `## Test Auth` absent + `TEST_AUTH_BEING_CREATED = false`.

## definition-of-done

### Feature: Health probe
- [ ] `GET /health` → 200, body `{"status":"ok","version":"0.1.0"}`.
- [ ] `version` field = exact `CARGO_PKG_VERSION` from `backend/Cargo.toml`.
- [ ] response Content-Type `application/json`.
- [ ] integration test `tests/health.rs::health_returns_ok` passes.

### Feature: List conversations
- [ ] `GET /api/conversations` → 200, body `[]` when DB empty.
- [ ] after one create: list contains row w/ matching `id` + `title`.
- [ ] order = `updated_at DESC` (most recent first).
- [ ] each item has `id` (UUID v4 string), `title`, `created_at`, `updated_at` (ISO-8601 millis).
- [ ] integration test `tests/conversations.rs::create_persists_and_list_returns` passes.

### Feature: Create conversation
- [ ] `POST /api/conversations` w/ `{"title":"Test"}` → 201.
- [ ] body `{"id":"<uuid-v4>","title":"Test","created_at":"<iso>","updated_at":"<iso>"}`.
- [ ] `id` parses as UUID v4 → `Uuid::parse_str` ok.
- [ ] `created_at == updated_at` on insert.
- [ ] empty title → 400 `error.code == "VALIDATION"`.
- [ ] title >200 chars → 400 `VALIDATION`.
- [ ] DB row inserted in `conversations` table → reachable via list.
- [ ] integration tests `tests/conversations.rs::{create_rejects_empty_title, create_rejects_too_long_title, create_returns_uuid_v4}` pass.

### Feature: List messages
- [ ] `GET /api/conversations/:id/messages` → 200 `[]` when no messages.
- [ ] missing conv → 404 `error.code == "NOT_FOUND"`.
- [ ] returns messages sorted by `created_at ASC`.
- [ ] each item has `id`, `conversation_id`, `role` (lowercase user|assistant|system), `content`, `created_at`.
- [ ] integration test `tests/messages.rs::list_messages_returns_in_order` passes.

### Feature: Send message (sync)
- [ ] `POST /api/conversations/:id/messages` w/ `{"content":"hi"}` → 201.
- [ ] body `{"user_message":{...},"assistant_message":{...}}`.
- [ ] both messages persisted in DB → list returns them.
- [ ] `user_message.role == "user"`, `assistant_message.role == "assistant"`.
- [ ] empty content → 400 `VALIDATION`.
- [ ] content >32_000 chars → 400 `VALIDATION`.
- [ ] missing conv → 404 `NOT_FOUND`.
- [ ] upstream mock 429 → 502 `error.code == "UPSTREAM"`.
- [ ] `conversations.updated_at` advances on send.
- [ ] integration tests `tests/messages.rs::{send_*}` pass.

### Feature: Stream message (SSE)
- [ ] `POST /api/conversations/:id/stream` w/ `{"content":"hi"}` → 200, Content-Type `text/event-stream`.
- [ ] event sequence: ≥1 `event: token` w/ `data: <chunk>`, then `event: done` w/ `data: {"message_id":"<uuid>"}`.
- [ ] persisted assistant msg content = concatenation of all token chunks.
- [ ] upstream error mid-stream → `event: error` w/ `data: {"code":"UPSTREAM","message":"..."}` → stream closes.
- [ ] integration test `tests/messages.rs::stream_emits_tokens_then_done` passes.

### Feature: OpenRouter client
- [ ] `HttpOpenRouterClient::chat` → returns content string on 2xx.
- [ ] 429/5xx → `OpenRouterError::HttpStatus(code)`.
- [ ] `stream` parses `data: <json>` lines, terminates on `data: [DONE]`.
- [ ] empty `delta.content` skipped.
- [ ] `From<OpenRouterError> for AppError` → `AppError::Upstream`.
- [ ] integration tests `tests/openrouter_mock.rs::*` pass.

### Feature: Verification gate
- [ ] `cd backend && cargo fmt --check` → exit 0.
- [ ] `cd backend && cargo clippy --all-targets -- -D warnings` → exit 0.
- [ ] `cd backend && cargo test` → all tests pass.
- [ ] `cd backend && cargo build --release` → exit 0.

## integration-tests

### test-module: tests/health.rs
- what: `GET /health` returns 200 + json shape via tower oneshot.
- location: `backend/tests/health.rs` + `mod common;` (file `backend/tests/common/mod.rs`).
- cases:
  - `health_returns_ok` → status 200, `body["status"] == "ok"`, `body["version"].is_string()`.
- references: `backendplan.md` §files `backend/tests/health.rs`.
- run-command: `cargo test --manifest-path backend/Cargo.toml --test health`.

### test-module: tests/conversations.rs
- what: validation + persistence + listing for `/api/conversations`.
- location: `backend/tests/conversations.rs`.
- cases:
  - `create_rejects_empty_title` → POST `{"title":""}` → 400, `body["error"]["code"] == "VALIDATION"`.
  - `create_rejects_too_long_title` → 201-char title → 400 VALIDATION.
  - `create_persists_and_list_returns` → POST valid → 201, then GET list → array length 1, item id matches.
  - `create_returns_uuid_v4` → POST valid → `Uuid::parse_str(body["id"]).is_ok()`.
- references: `backendplan.md` §files `backend/tests/conversations.rs`.
- run-command: `cargo test --manifest-path backend/Cargo.toml --test conversations`.

### test-module: tests/messages.rs
- what: send (sync) + stream w/ MockOpenRouterClient.
- location: `backend/tests/messages.rs`.
- cases:
  - `send_rejects_empty_content` → POST `{"content":""}` → 400 VALIDATION.
  - `send_rejects_too_long_content` → 32_001-char content → 400 VALIDATION.
  - `send_returns_user_and_assistant` → mock chat returns `"Hello back"` → 201, body has both messages, assistant content `"Hello back"`.
  - `send_404_when_conv_missing` → unknown UUID → 404 NOT_FOUND.
  - `send_502_when_upstream_fails` → mock w/ `OpenRouterError::HttpStatus(429)` → 502 UPSTREAM.
  - `list_messages_returns_in_order` → seed 3 messages → GET returns sorted by created_at asc.
  - `stream_emits_tokens_then_done` → mock stream `["Hel","lo"]` → SSE body parsed → events `token:"Hel"`, `token:"lo"`, `done` w/ `message_id` UUID. assistant DB row content == `"Hello"`.
- references: `backendplan.md` §files `backend/tests/messages.rs`.
- run-command: `cargo test --manifest-path backend/Cargo.toml --test messages`.

### test-module: tests/openrouter_mock.rs
- what: `HttpOpenRouterClient` HTTP + stream parsing via mockito.
- location: `backend/tests/openrouter_mock.rs`.
- cases:
  - `http_chat_success` → mockito 200 `{"choices":[{"message":{"content":"hi"}}]}` → `client.chat()` → `"hi"`.
  - `http_chat_429_maps_to_http_status` → mockito 429 → `Err(OpenRouterError::HttpStatus(429))`.
  - `http_chat_5xx_maps_to_http_status` → mockito 503 → `Err(OpenRouterError::HttpStatus(503))`.
  - `http_chat_decode_failure` → mockito 200 `{}` (no choices) → `Err(OpenRouterError::Decode(_))`.
  - `http_stream_parses_data_lines` → mockito event-stream body `data: {"choices":[{"delta":{"content":"Hel"}}]}\n\ndata: {"choices":[{"delta":{"content":"lo"}}]}\n\ndata: [DONE]\n\n` → collected = `["Hel","lo"]`.
  - `http_stream_skips_empty_delta` → chunk w/ `delta:{}` → emits no item, continues.
  - `http_stream_terminates_on_done` → trailing bytes after `[DONE]` ignored, stream ends.
- references: `backendplan.md` §files `backend/tests/openrouter_mock.rs`.
- run-command: `cargo test --manifest-path backend/Cargo.toml --test openrouter_mock`.

### test-module: tests/common/mod.rs
- what: shared helper module (not a test).
- location: `backend/tests/common/mod.rs`.
- contents: `test_app() -> Router`, `test_app_with_mock(MockOpenRouterClient) -> Router`, `read_json(Response) -> serde_json::Value`, `json_body(Value) -> Body`. tempfile-backed SQLite DB per call. Config w/ stub api key.
- references: `backendplan.md` §files `backend/tests/common/mod.rs`.

## api-tests

Notes for all groups:
- base URL: `http://localhost:8080`.
- no auth → no `Authorization` header.
- backend MUST run w/ `BACKEND_USE_MOCK_OPENROUTER=1` so upstream calls return canned responses (mock yields chat `"Hello back"`, stream `["Hel","lo"]`, error variant when env `BACKEND_MOCK_FAIL=1`).
- variables: shell vars `CONV_ID` (extracted via jq), `BAD_ID="00000000-0000-0000-0000-000000000000"`.
- order matters within each group: setup steps create state used by later assertions. cleanup not required (run against fresh DB or accept residue — `qa run --fresh` removes `gpt-copy-v8.sqlite` between runs).
- response time budget: CRUD 500ms, joins 1000ms, stream first-token 2000ms.

### test-group: health
- endpoint: GET /health
- setup: backend up; no DB rows needed.
- validation-steps:
  1. description: "liveness probe returns ok"
     curl: `curl -s -o /tmp/_qa.json -w '%{http_code}' http://localhost:8080/health`
     expect-status: 200
     expect-body-contains: ["status", "version"]
     expect-body-shape: {"status": "ok", "version": "0.1.0"}
     expect-response-time: 200ms
- notes: stateless. run first → confirms server up before other groups.

### test-group: conversations-list-empty
- endpoint: GET /api/conversations
- setup: fresh DB (no rows).
- validation-steps:
  1. description: "empty list on fresh DB"
     curl: `curl -s -o /tmp/_qa.json -w '%{http_code}' http://localhost:8080/api/conversations`
     expect-status: 200
     expect-body-shape: []
     expect-response-time: 500ms
- notes: must run before any create step. parse: `jq 'length == 0' /tmp/_qa.json` → true.

### test-group: conversations-create
- endpoint: POST /api/conversations
- setup: fresh DB.
- validation-steps:
  1. description: "create succeeds w/ valid title"
     curl: `curl -s -o /tmp/_qa.json -w '%{http_code}' -X POST http://localhost:8080/api/conversations -H 'Content-Type: application/json' -d '{"title":"Test"}'`
     expect-status: 201
     expect-body-contains: ["id", "title", "created_at", "updated_at"]
     expect-body-shape: {"id":"<uuid>","title":"Test","created_at":"<iso8601>","updated_at":"<iso8601>"}
     expect-response-time: 500ms
     post-step: `CONV_ID=$(jq -r .id /tmp/_qa.json)` → save for later groups.
  2. description: "list now contains created conv"
     curl: `curl -s -o /tmp/_qa.json -w '%{http_code}' http://localhost:8080/api/conversations`
     expect-status: 200
     expect-body-contains: ["Test"]
     expect-body-shape: array length >= 1, first item id == $CONV_ID
     expect-response-time: 500ms
  3. description: "rejects empty title → 400 VALIDATION"
     curl: `curl -s -o /tmp/_qa.json -w '%{http_code}' -X POST http://localhost:8080/api/conversations -H 'Content-Type: application/json' -d '{"title":""}'`
     expect-status: 400
     expect-body-contains: ["error", "VALIDATION"]
     expect-body-shape: {"error":{"code":"VALIDATION","message":"<non-empty>"}}
     expect-response-time: 200ms
  4. description: "rejects 201-char title → 400 VALIDATION"
     curl: `curl -s -o /tmp/_qa.json -w '%{http_code}' -X POST http://localhost:8080/api/conversations -H 'Content-Type: application/json' -d "{\"title\":\"$(printf 'a%.0s' {1..201})\"}"`
     expect-status: 400
     expect-body-contains: ["error", "VALIDATION"]
     expect-response-time: 300ms
  5. description: "rejects missing title field → 400 VALIDATION (or 422 deserialize)"
     curl: `curl -s -o /tmp/_qa.json -w '%{http_code}' -X POST http://localhost:8080/api/conversations -H 'Content-Type: application/json' -d '{}'`
     expect-status: 400
     expect-body-contains: ["error"]
     expect-response-time: 200ms
  6. description: "rejects very long title (10_000 chars) → 400, no 500"
     curl: `curl -s -o /tmp/_qa.json -w '%{http_code}' -X POST http://localhost:8080/api/conversations -H 'Content-Type: application/json' -d "{\"title\":\"$(printf 'a%.0s' {1..10000})\"}"`
     expect-status: 400
     expect-body-contains: ["error", "VALIDATION"]
     expect-response-time: 500ms
  7. description: "injection payload sanitized → 201 (stored verbatim, NOT executed) OR 400"
     curl: `curl -s -o /tmp/_qa.json -w '%{http_code}' -X POST http://localhost:8080/api/conversations -H 'Content-Type: application/json' -d '{"title":"<script>alert(1)</script>"}'`
     expect-status: 201
     expect-body-contains: ["title"]
     expect-body-shape: {"title":"<script>alert(1)</script>"} verbatim (sqlite parameterized → no SQL injection; XSS = frontend concern).
     expect-response-time: 500ms
  8. description: "SQL injection attempt → stored verbatim, no DB damage"
     curl: `curl -s -o /tmp/_qa.json -w '%{http_code}' -X POST http://localhost:8080/api/conversations -H 'Content-Type: application/json' -d "{\"title\":\"' OR 1=1; --\"}"`
     expect-status: 201
     expect-body-shape: {"title":"' OR 1=1; --"}
     expect-response-time: 500ms
- notes: order matters — step 1 sets `$CONV_ID` used downstream. error responses MUST be JSON `{"error":{"code":"...","message":"..."}}` — never stack traces.

### test-group: messages-list
- endpoint: GET /api/conversations/:id/messages
- setup: $CONV_ID from `conversations-create` step 1.
- validation-steps:
  1. description: "empty messages list for new conv"
     curl: `curl -s -o /tmp/_qa.json -w '%{http_code}' http://localhost:8080/api/conversations/$CONV_ID/messages`
     expect-status: 200
     expect-body-shape: []
     expect-response-time: 500ms
  2. description: "missing conv → 404 NOT_FOUND"
     curl: `curl -s -o /tmp/_qa.json -w '%{http_code}' http://localhost:8080/api/conversations/00000000-0000-0000-0000-000000000000/messages`
     expect-status: 404
     expect-body-contains: ["error", "NOT_FOUND"]
     expect-body-shape: {"error":{"code":"NOT_FOUND","message":"<non-empty>"}}
     expect-response-time: 300ms
  3. description: "malformed UUID → 400 (axum Path extractor)"
     curl: `curl -s -o /tmp/_qa.json -w '%{http_code}' http://localhost:8080/api/conversations/not-a-uuid/messages`
     expect-status: 400
     expect-body-contains: ["error"]
     expect-response-time: 200ms
- notes: depends on `conversations-create` having run.

### test-group: messages-send
- endpoint: POST /api/conversations/:id/messages
- setup: $CONV_ID from `conversations-create`. backend run w/ `BACKEND_USE_MOCK_OPENROUTER=1` → mock returns `"Hello back"`.
- validation-steps:
  1. description: "send valid content → 201 + user+assistant pair"
     curl: `curl -s -o /tmp/_qa.json -w '%{http_code}' -X POST http://localhost:8080/api/conversations/$CONV_ID/messages -H 'Content-Type: application/json' -d '{"content":"hi"}'`
     expect-status: 201
     expect-body-contains: ["user_message", "assistant_message"]
     expect-body-shape: {"user_message":{"id":"<uuid>","conversation_id":"<uuid>","role":"user","content":"hi","created_at":"<iso>"},"assistant_message":{"id":"<uuid>","conversation_id":"<uuid>","role":"assistant","content":"Hello back","created_at":"<iso>"}}
     expect-response-time: 1000ms
  2. description: "list now returns 2 messages in created order"
     curl: `curl -s -o /tmp/_qa.json -w '%{http_code}' http://localhost:8080/api/conversations/$CONV_ID/messages`
     expect-status: 200
     expect-body-contains: ["user", "assistant", "Hello back"]
     expect-body-shape: array length 2, [0].role=="user", [1].role=="assistant"
     expect-response-time: 500ms
  3. description: "empty content → 400 VALIDATION"
     curl: `curl -s -o /tmp/_qa.json -w '%{http_code}' -X POST http://localhost:8080/api/conversations/$CONV_ID/messages -H 'Content-Type: application/json' -d '{"content":""}'`
     expect-status: 400
     expect-body-contains: ["error", "VALIDATION"]
     expect-response-time: 200ms
  4. description: "content >32_000 chars → 400 VALIDATION"
     curl: `curl -s -o /tmp/_qa.json -w '%{http_code}' -X POST http://localhost:8080/api/conversations/$CONV_ID/messages -H 'Content-Type: application/json' -d "{\"content\":\"$(printf 'a%.0s' {1..32001})\"}"`
     expect-status: 400
     expect-body-contains: ["error", "VALIDATION"]
     expect-response-time: 1000ms
  5. description: "missing content field → 400 (or 422 deserialize)"
     curl: `curl -s -o /tmp/_qa.json -w '%{http_code}' -X POST http://localhost:8080/api/conversations/$CONV_ID/messages -H 'Content-Type: application/json' -d '{}'`
     expect-status: 400
     expect-body-contains: ["error"]
     expect-response-time: 200ms
  6. description: "missing conv → 404 NOT_FOUND"
     curl: `curl -s -o /tmp/_qa.json -w '%{http_code}' -X POST http://localhost:8080/api/conversations/00000000-0000-0000-0000-000000000000/messages -H 'Content-Type: application/json' -d '{"content":"hi"}'`
     expect-status: 404
     expect-body-contains: ["error", "NOT_FOUND"]
     expect-response-time: 500ms
  7. description: "upstream mock failure → 502 UPSTREAM"
     setup: restart backend w/ `BACKEND_MOCK_FAIL=1` → mock returns `OpenRouterError::HttpStatus(429)`.
     curl: `curl -s -o /tmp/_qa.json -w '%{http_code}' -X POST http://localhost:8080/api/conversations/$CONV_ID/messages -H 'Content-Type: application/json' -d '{"content":"hi"}'`
     expect-status: 502
     expect-body-contains: ["error", "UPSTREAM"]
     expect-body-shape: {"error":{"code":"UPSTREAM","message":"<non-empty>"}}
     expect-response-time: 1000ms
  8. description: "injection in content stored verbatim"
     curl: `curl -s -o /tmp/_qa.json -w '%{http_code}' -X POST http://localhost:8080/api/conversations/$CONV_ID/messages -H 'Content-Type: application/json' -d "{\"content\":\"' OR 1=1; --\"}"`
     expect-status: 201
     expect-body-contains: ["user_message"]
     expect-response-time: 1000ms
- notes: step 7 needs backend restart w/ different env. group runs after `conversations-create`. error bodies MUST be JSON, never stack traces.

### test-group: messages-stream
- endpoint: POST /api/conversations/:id/stream
- setup: $CONV_ID. backend up w/ mock stream → yields `["Hel","lo"]`.
- validation-steps:
  1. description: "stream returns SSE events token×N then done"
     curl: `curl -s -N -o /tmp/_qa.sse -w '%{http_code}\n%{content_type}' -X POST http://localhost:8080/api/conversations/$CONV_ID/stream -H 'Content-Type: application/json' -d '{"content":"hi"}'`
     expect-status: 200
     expect-content-type: "text/event-stream"
     expect-body-contains: ["event: token", "data: Hel", "data: lo", "event: done", "message_id"]
     expect-body-shape: SSE frames separated by `\n\n`. final frame `event: done\ndata: {"message_id":"<uuid>"}`.
     expect-response-time: 2000ms (first byte), total stream <5000ms for mock.
  2. description: "after stream done, messages list contains assistant w/ content 'Hello'"
     curl: `curl -s -o /tmp/_qa.json -w '%{http_code}' http://localhost:8080/api/conversations/$CONV_ID/messages`
     expect-status: 200
     expect-body-contains: ["Hello"]
     expect-body-shape: assistant message content = concatenation `"Hello"` (chunks `"Hel"+"lo"`).
     expect-response-time: 500ms
  3. description: "upstream mock stream error → SSE event: error"
     setup: backend env `BACKEND_MOCK_STREAM_FAIL=1` → mock stream errors on second chunk.
     curl: `curl -s -N -o /tmp/_qa.sse -w '%{http_code}' -X POST http://localhost:8080/api/conversations/$CONV_ID/stream -H 'Content-Type: application/json' -d '{"content":"hi"}'`
     expect-status: 200
     expect-body-contains: ["event: token", "event: error", "UPSTREAM"]
     expect-response-time: 2000ms
  4. description: "missing conv on stream → 404 NOT_FOUND"
     curl: `curl -s -o /tmp/_qa.json -w '%{http_code}' -X POST http://localhost:8080/api/conversations/00000000-0000-0000-0000-000000000000/stream -H 'Content-Type: application/json' -d '{"content":"hi"}'`
     expect-status: 404
     expect-body-contains: ["error", "NOT_FOUND"]
     expect-response-time: 500ms
  5. description: "empty content → 400 VALIDATION"
     curl: `curl -s -o /tmp/_qa.json -w '%{http_code}' -X POST http://localhost:8080/api/conversations/$CONV_ID/stream -H 'Content-Type: application/json' -d '{"content":""}'`
     expect-status: 400
     expect-body-contains: ["error", "VALIDATION"]
     expect-response-time: 200ms
- notes: SSE response → use `curl -N` (no buffering). parse `/tmp/_qa.sse` w/ grep for event lines. step 3 needs backend restart.

## test-environment

- startup:
    method: local (cargo)
    command: `OPENROUTER_API_KEY=test BACKEND_USE_MOCK_OPENROUTER=1 cargo run --manifest-path backend/Cargo.toml --bin server`
    base-url: `http://localhost:8080`
    health-check: `http://localhost:8080/health` → 200 `{"status":"ok",...}`
    reference: `backend/CLAUDE.md` (created during impl) → "## Running" section. Bootstrap: no CLAUDE.md yet → use this qaplan.
- seed-data: none. fresh empty SQLite DB per run.
- auth: none.
- env-vars:
  - `OPENROUTER_API_KEY` (required by `Config::from_env` → fails fast if missing; value `test` accepted when mock active).
  - `BACKEND_USE_MOCK_OPENROUTER=1` (swap `HttpOpenRouterClient` → `MockOpenRouterClient` in `main.rs` for API tests).
  - `BACKEND_MOCK_FAIL=1` (optional → mock chat returns `OpenRouterError::HttpStatus(429)` → drives 502 UPSTREAM test).
  - `BACKEND_MOCK_STREAM_FAIL=1` (optional → mock stream errors on second chunk → drives SSE error test).
  - `DATABASE_URL` (default `sqlite://backend/gpt-copy-v8.sqlite?mode=rwc`).
  - `BACKEND_PORT=8080` (default).
  - `BACKEND_HOST=127.0.0.1` (default).
  - `FRONTEND_ORIGIN=http://localhost:3000` (default).
- cleanup: delete `backend/gpt-copy-v8.sqlite` before each `qa run --fresh`. integration tests use tempfile DBs → no cleanup.
- container: not required. Rust tests run on host via `cargo test`.
- impl note: `BACKEND_USE_MOCK_OPENROUTER` env switch MUST be wired in `backend/src/main.rs` step 5 — when set, swap `HttpOpenRouterClient::new(...)` → `MockOpenRouterClient::with_chat("Hello back")` (or stream variant). flag this back to backend impl agent if not in plan.

QAPLAN_COMPLETE: backendqaplan.md
