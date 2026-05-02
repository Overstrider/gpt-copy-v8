# arcplan

## meta
- project: gpt-copy-v8
- task: bootstrap ChatGPT-style monorepo. Rust+Axum backend + Next.js+TS frontend. OpenRouter proxied server-side.
- repos: backend, frontend
- lang: backend=Rust 2024, frontend=TypeScript 5
- stack: backend=Axum 0.7+sqlx+SQLite+tokio+reqwest+tracing, frontend=Next.js 15 App Router+Tailwind 3+TanStack Query+zod+react-markdown
- mode: bootstrap
- execution-order: backend → frontend
- PROJECT_RULES_STATUS: approved
- PROJECT_RULES_DIGEST: 54eae19e90a1b403912baee95e0294f1ff7bdf2aff6afb8d07af8d0744677dfe
- PROJECT_RULES_READ: yes

## Project Description

ChatGPT-style chat UI. Backend persists conversations + messages in SQLite, proxies OpenRouter completions (sync + streaming). Frontend renders sidebar of past conversations, transcript with markdown bubbles, composer with streaming token render. No client-side OpenRouter calls. Secrets stay in untracked `.env`.

## Stack & Dependencies

### backend (Cargo.toml)
- `axum` 0.7 → HTTP framework, SSE support built-in.
- `tokio` 1 features=["full"] → async runtime.
- `tower-http` 0.5 features=["cors","trace"] → CORS + tracing middleware.
- `sqlx` 0.8 features=["runtime-tokio-rustls","sqlite","uuid","chrono","macros","migrate"] → DB + queries + migrations.
- `serde` 1 features=["derive"], `serde_json` 1 → JSON.
- `reqwest` 0.12 features=["json","stream","rustls-tls"] default-features=false → OpenRouter HTTP client.
- `futures-util` 0.3 → stream combinators for SSE relay.
- `tokio-stream` 0.1 → stream adapters.
- `uuid` 1 features=["v4","serde"] → conversation/message IDs.
- `chrono` 0.4 features=["serde"] → timestamps.
- `tracing` 0.1 + `tracing-subscriber` 0.3 features=["env-filter","fmt"] → structured logs.
- `thiserror` 1 → error enum.
- `dotenvy` 0.15 → load `.env` at startup.
- `async-trait` 0.1 → trait OpenRouterClient for mocking.
- dev-deps: `mockito` 1 (HTTP mock for OpenRouter tests), `tower` 0.5 features=["util"] (oneshot for handler tests), `tempfile` 3 (sqlite test DB).

### frontend (package.json)
- `next` ^15, `react` ^19, `react-dom` ^19.
- `typescript` ^5, `@types/react`, `@types/node`.
- `tailwindcss` ^3, `postcss`, `autoprefixer`.
- `@tanstack/react-query` ^5 → cache list + messages.
- `zod` ^3 → validate API responses.
- `lucide-react` → icons (Plus, Send, Menu, AlertCircle, Loader).
- `react-markdown` ^9 + `remark-gfm` ^4 → render assistant markdown safely.
- `clsx` → conditional classes.
- dev: `vitest` ^2, `@testing-library/react`, `@testing-library/jest-dom`, `jsdom`, `@vitejs/plugin-react`.
- dev: `@playwright/test` ^1 → smoke E2E.
- dev: `eslint`, `eslint-config-next`, `prettier`.

## Directory Structure

```
gpt-copy-v8/
├── .env.example                 (exists, keep)
├── .gitignore                   (exists, keep)
├── README.md                    (rewrite: setup, env, run, test, troubleshoot)
├── backend/
│   ├── Cargo.toml
│   ├── Cargo.lock
│   ├── rust-toolchain.toml      (channel="stable", edition uses 2024)
│   ├── migrations/
│   │   └── 0001_init.sql
│   ├── src/
│   │   ├── main.rs              (bootstrap: env, tracing, db, router, listen)
│   │   ├── lib.rs               (re-export modules + build_app(state) for tests)
│   │   ├── config.rs            (Config struct, from_env)
│   │   ├── error.rs             (AppError enum + IntoResponse → JSON)
│   │   ├── state.rs             (AppState { pool, openrouter: Arc<dyn OpenRouterClient> })
│   │   ├── db.rs                (init_pool, run_migrations)
│   │   ├── models.rs            (Conversation, Message, Role)
│   │   ├── openrouter.rs        (trait OpenRouterClient, HttpOpenRouterClient, MockOpenRouterClient)
│   │   └── routes/
│   │       ├── mod.rs           (router fn aggregating all routes)
│   │       ├── health.rs        (GET /health)
│   │       ├── conversations.rs (list + create)
│   │       └── messages.rs      (list messages, send, stream)
│   └── tests/
│       ├── health.rs
│       ├── conversations.rs
│       ├── messages.rs
│       └── openrouter_mock.rs
└── frontend/
    ├── package.json
    ├── tsconfig.json
    ├── next.config.ts
    ├── tailwind.config.ts
    ├── postcss.config.mjs
    ├── .eslintrc.json
    ├── playwright.config.ts
    ├── vitest.config.ts
    ├── vitest.setup.ts
    ├── public/
    ├── e2e/
    │   └── send-message.spec.ts
    └── src/
        ├── app/
        │   ├── layout.tsx       (html shell, providers)
        │   ├── page.tsx         (chat shell: Sidebar + Main)
        │   ├── globals.css      (tailwind directives)
        │   └── providers.tsx    (QueryClientProvider)
        ├── components/
        │   ├── Sidebar.tsx
        │   ├── ConversationItem.tsx
        │   ├── ChatWindow.tsx
        │   ├── MessageList.tsx
        │   ├── MessageBubble.tsx
        │   ├── Markdown.tsx
        │   ├── Composer.tsx
        │   ├── EmptyState.tsx
        │   ├── ErrorBanner.tsx
        │   └── LoadingDots.tsx
        ├── hooks/
        │   ├── useConversations.ts
        │   ├── useMessages.ts
        │   ├── useCreateConversation.ts
        │   └── useStreamMessage.ts
        ├── lib/
        │   ├── api.ts           (fetch wrapper + API_BASE)
        │   ├── schemas.ts       (zod schemas)
        │   └── types.ts         (TS types from zod infer)
        └── __tests__/
            ├── MessageBubble.test.tsx
            ├── Composer.test.tsx
            └── Sidebar.test.tsx
```

## Module Guide

### backend
- `config.rs` → loads OPENROUTER_API_KEY, OPENROUTER_MODEL, DATABASE_URL, BACKEND_HOST, BACKEND_PORT, FRONTEND_ORIGIN. Fails fast on missing keys.
- `error.rs` → `AppError` enum (Validation, NotFound, Database, Upstream, Internal). `IntoResponse` returns `{ "error": { "code": str, "message": str } }` + matching status.
- `state.rs` → `AppState` cloned per req. Holds `SqlitePool` + `Arc<dyn OpenRouterClient>`. Trait object → mock in tests.
- `db.rs` → `init_pool(url)` runs `sqlx::migrate!("./migrations")`.
- `models.rs` → DB rows + DTOs.
- `openrouter.rs` → trait with two fns: `chat(req)` returns full message; `stream(req)` returns `Stream<Item=Result<String, Error>>` of token deltas. `HttpOpenRouterClient` calls `https://openrouter.ai/api/v1/chat/completions` with bearer key. `MockOpenRouterClient` (test-only) returns canned strings.
- `routes/*` → handlers only. Validate input, call DB + OpenRouter via state, return JSON or SSE.

### frontend
- `app/page.tsx` → server component shell. Renders `<Providers><Sidebar/><ChatWindow/></Providers>`. State lifted via QueryClient + URL query (`?c=<id>`).
- `Sidebar` → list conversations via `useConversations()`. Highlights active. "New chat" button calls `useCreateConversation`. Mobile: drawer toggled by Menu icon.
- `ChatWindow` → reads active conversation ID. Renders `MessageList` + `Composer`. Shows `EmptyState` when none selected.
- `MessageList` → maps messages → `MessageBubble`. Auto-scroll on new content.
- `MessageBubble` → user vs assistant styling. Assistant uses `Markdown` component. Loading state shows `LoadingDots` while streaming.
- `Markdown` → wraps `react-markdown` with `remark-gfm`. Disallow raw HTML via `react-markdown` default config (skipHtml).
- `Composer` → textarea + Send button. Enter=send, Shift+Enter=newline. Disabled while streaming.
- `useConversations` → TanStack Query GET list. zod-validated.
- `useMessages` → TanStack Query GET messages by conv id.
- `useCreateConversation` → mutation POST.
- `useStreamMessage` → native `fetch` with `ReadableStream` reader on `/api/conversations/:id/stream`. Parses SSE `data:` lines, appends tokens to local state, invalidates `useMessages` on done.
- `lib/api.ts` → `apiFetch<T>(path, init, schema)` → throws structured error matching backend shape.
- `lib/schemas.ts` → zod for Conversation, Message, ErrorBody.

## repo:backend

### scope
Bootstrap Rust 2024 Axum API in `backend/`. Persists conversations + messages in SQLite via sqlx. Exposes REST + SSE endpoints. Proxies OpenRouter server-side. Loads OPENROUTER_API_KEY + OPENROUTER_MODEL from env. Tests cover health, validation, DB persistence, mocked OpenRouter behavior.

### prerequisite:project-rules
- PROJECT_RULES_STATUS: approved
- PROJECT_RULES_DIGEST: 54eae19e90a1b403912baee95e0294f1ff7bdf2aff6afb8d07af8d0744677dfe
- PROJECT_RULES_READ: yes
- MUST keep OpenRouter calls server-side only.
- MUST default OPENROUTER_MODEL to `nvidia/nemotron-3-super-120b-a12b:free`.
- MUST NOT commit secrets. `.env` ignored, `.env.example` placeholder-only.
- MUST handle OpenRouter 429, timeout, invalid response, interrupted stream without crash.
- MUST test provider failures with mocks. No real OpenRouter calls in tests.
- VERIFY with `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`.

### affected-modules
- module: `backend/Cargo.toml` — action: create — reason: crate manifest, edition="2024", deps listed in Stack.
- module: `backend/rust-toolchain.toml` — action: create — reason: pin stable, ensure 2024 edition support.
- module: `backend/migrations/0001_init.sql` — action: create — reason: schema for conversations + messages.
- module: `backend/src/main.rs` — action: create — reason: entry: load env, init tracing+DB+client, build router, bind host:port.
- module: `backend/src/lib.rs` — action: create — reason: expose `build_app(state) -> Router` so tests reuse without binding socket.
- module: `backend/src/config.rs` — action: create — reason: env parsing, single source of truth.
- module: `backend/src/error.rs` — action: create — reason: `AppError` + JSON `IntoResponse`.
- module: `backend/src/state.rs` — action: create — reason: shared `AppState` for handlers.
- module: `backend/src/db.rs` — action: create — reason: pool init + migration runner.
- module: `backend/src/models.rs` — action: create — reason: row + DTO types.
- module: `backend/src/openrouter.rs` — action: create — reason: trait + HTTP impl + Mock impl.
- module: `backend/src/routes/mod.rs` — action: create — reason: aggregate routers, attach CORS + trace layers.
- module: `backend/src/routes/health.rs` — action: create — reason: liveness probe.
- module: `backend/src/routes/conversations.rs` — action: create — reason: list + create.
- module: `backend/src/routes/messages.rs` — action: create — reason: list + send + stream.
- module: `backend/tests/health.rs` — action: create — reason: GET /health → 200.
- module: `backend/tests/conversations.rs` — action: create — reason: validation + persistence.
- module: `backend/tests/messages.rs` — action: create — reason: send/stream w/ mock.
- module: `backend/tests/openrouter_mock.rs` — action: create — reason: trait impl behavior + error paths.

### requirements
1. Cargo manifest declares `edition = "2024"`. Crate name `gpt_copy_v8_backend`. `[lib]` + `[[bin]] name="server" path="src/main.rs"`.
2. Migration `0001_init.sql` creates two tables:
   ```sql
   CREATE TABLE conversations (
     id TEXT PRIMARY KEY,
     title TEXT NOT NULL,
     created_at TEXT NOT NULL,
     updated_at TEXT NOT NULL
   );
   CREATE TABLE messages (
     id TEXT PRIMARY KEY,
     conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
     role TEXT NOT NULL CHECK(role IN ('user','assistant','system')),
     content TEXT NOT NULL,
     created_at TEXT NOT NULL
   );
   CREATE INDEX idx_messages_conversation ON messages(conversation_id, created_at);
   ```
3. `Config` reads env: `OPENROUTER_API_KEY` (required), `OPENROUTER_MODEL` (default `nvidia/nemotron-3-super-120b-a12b:free`), `DATABASE_URL` (default `sqlite://backend/gpt-copy-v8.sqlite?mode=rwc`), `BACKEND_HOST` (default `127.0.0.1`), `BACKEND_PORT` (default `8080`), `FRONTEND_ORIGIN` (default `http://localhost:3000`).
4. `AppError` variants → status codes: `Validation` 400, `NotFound` 404, `Upstream` 502, `Database` 500, `Internal` 500. Body shape `{ "error": { "code": "VALIDATION", "message": "<msg>" } }`.
5. `OpenRouterClient` trait: `async fn chat(&self, model: &str, messages: Vec<ChatMessage>) -> Result<String, OpenRouterError>` + `async fn stream(&self, model: &str, messages: Vec<ChatMessage>) -> Result<BoxStream<'static, Result<String, OpenRouterError>>, OpenRouterError>`. `HttpOpenRouterClient` posts to `https://openrouter.ai/api/v1/chat/completions` with `Authorization: Bearer <key>`. Streaming uses SSE `data: {...}` parsing of `choices[0].delta.content`. Stops on `data: [DONE]`.
6. CORS layer allows `FRONTEND_ORIGIN` only, methods GET/POST/OPTIONS, headers content-type, credentials false.
7. Tracing: `tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))`.
8. SSE handler uses `axum::response::sse::Sse` with `KeepAlive`. Each event: `event: token` + `data: <text>`. Final event `event: done` + `data: ok`. On error: `event: error` + `data: <message>`. Persist assistant message after stream completes (or on disconnect with partial content).
9. Validation: title ≤ 200 chars non-empty. Message content ≤ 32_000 chars non-empty. Conversation id valid UUID v4.
10. Tests use `build_app(state)` + `tower::ServiceExt::oneshot`. SQLite DB → tempfile per test. `MockOpenRouterClient` returns deterministic strings + injectable errors.
11. Use `sqlx::query` (runtime) not `query!` macro → avoid offline metadata complexity. Prime Directive: simplicity.
12. `.env.example` exists at repo root → backend reads via `dotenvy::from_path("../.env").ok()` then falls back to process env.

### interfaces
- exposes: `GET /health` (see cross-repo).
- exposes: `GET /api/conversations` (see cross-repo).
- exposes: `POST /api/conversations` (see cross-repo).
- exposes: `GET /api/conversations/:id/messages` (see cross-repo).
- exposes: `POST /api/conversations/:id/messages` (see cross-repo).
- exposes: `POST /api/conversations/:id/stream` (SSE, see cross-repo).

## repo:frontend

### scope
Bootstrap Next.js 15 App Router app in `frontend/` w/ TS + Tailwind. ChatGPT-style UI: sidebar conversations, transcript, composer, markdown assistant bubbles, streaming. zod validates API. TanStack Query caches list + history. Native fetch + ReadableStream handles SSE. Vitest + Testing Library for components. One Playwright smoke spec for sending a message.

### prerequisite:project-rules
- PROJECT_RULES_STATUS: approved
- PROJECT_RULES_DIGEST: 54eae19e90a1b403912baee95e0294f1ff7bdf2aff6afb8d07af8d0744677dfe
- PROJECT_RULES_READ: yes
- MUST NOT call OpenRouter from client. Always proxy via backend.
- MUST NOT commit secrets. Only `NEXT_PUBLIC_API_BASE_URL` exposed to client.
- MUST validate every backend response w/ zod.
- MUST render assistant markdown via `react-markdown` + `remark-gfm`. Forbid raw-HTML injection sinks; use `react-markdown` `skipHtml` default.
- VERIFY with `npm run lint`, `npm run test`, `npm run build`, `npm run test:e2e`.

### affected-modules
- module: `frontend/package.json` — action: create — reason: deps + scripts.
- module: `frontend/tsconfig.json` — action: create — reason: strict TS + path alias `@/*`.
- module: `frontend/next.config.ts` — action: create — reason: minimal config, react strict.
- module: `frontend/tailwind.config.ts` — action: create — reason: scan `./src/**/*.{ts,tsx}`.
- module: `frontend/postcss.config.mjs` — action: create — reason: tailwind+autoprefixer plugins.
- module: `frontend/playwright.config.ts` — action: create — reason: webServer `npm run dev`, baseURL `http://localhost:3000`.
- module: `frontend/vitest.config.ts` + `vitest.setup.ts` — action: create — reason: jsdom env + jest-dom matchers.
- module: `frontend/.eslintrc.json` — action: create — reason: extends `next/core-web-vitals`.
- module: `frontend/src/app/layout.tsx` — action: create — reason: html shell + Providers.
- module: `frontend/src/app/page.tsx` — action: create — reason: chat shell composing Sidebar + ChatWindow.
- module: `frontend/src/app/providers.tsx` — action: create — reason: QueryClientProvider.
- module: `frontend/src/app/globals.css` — action: create — reason: tailwind directives + base layer.
- module: `frontend/src/components/*.tsx` — action: create — reason: UI primitives listed in Module Guide.
- module: `frontend/src/hooks/*.ts` — action: create — reason: data hooks.
- module: `frontend/src/lib/{api,schemas,types}.ts` — action: create — reason: API client + validation.
- module: `frontend/src/__tests__/*.test.tsx` — action: create — reason: Vitest component tests.
- module: `frontend/e2e/send-message.spec.ts` — action: create — reason: Playwright smoke.

### requirements
1. `package.json` scripts: `dev`=`next dev -p 3000`, `build`=`next build`, `start`=`next start -p 3000`, `lint`=`next lint`, `test`=`vitest run`, `test:watch`=`vitest`, `test:e2e`=`playwright test`, `test:e2e:install`=`playwright install --with-deps chromium`.
2. `tsconfig.json` strict=true, paths `{ "@/*": ["./src/*"] }`.
3. Tailwind palette neutral/zinc. Layout: full-viewport flex, sidebar 260px desktop, hidden behind drawer < md breakpoint.
4. State lives in TanStack Query cache + URL search param `?c=<conversationId>` for active conversation. No Redux/Zustand.
5. zod schemas in `lib/schemas.ts`:
   - `Conversation = { id: string.uuid(), title: string, created_at: string, updated_at: string }`
   - `Message = { id: string.uuid(), conversation_id: string.uuid(), role: enum['user','assistant','system'], content: string, created_at: string }`
   - `ErrorBody = { error: { code: string, message: string } }`
6. `lib/api.ts` reads `process.env.NEXT_PUBLIC_API_BASE_URL` (fallback `http://localhost:8080`). `apiFetch` parses error body via zod → throws `ApiError`.
7. Streaming: `useStreamMessage` opens POST to `/api/conversations/:id/stream`, reads `response.body!.getReader()`, parses `event:` + `data:` lines, dispatches token append. Aborts on unmount via `AbortController`. On `event: done` → invalidates `messages` query.
8. `MessageBubble` user variant: right-aligned, zinc-700 bg. Assistant variant: left-aligned, zinc-100 bg, renders `<Markdown>`. Streaming bubble shows pulsing cursor.
9. Loading state: `<LoadingDots/>` 3 dots animated during `isPending`. Error state: `<ErrorBanner message={err}/>` red bar w/ `AlertCircle` icon + retry button.
10. Vitest tests: MessageBubble renders user vs assistant correctly + markdown bold; Composer Enter sends + Shift+Enter newline + disabled state; Sidebar shows items + highlights active.
11. Playwright `e2e/send-message.spec.ts`: stub backend via `page.route('**/api/**', ...)` → mock list, create, stream (return SSE response) → assert assistant bubble appears w/ streamed text. Avoids real backend.
12. Frontend reads `NEXT_PUBLIC_API_BASE_URL` from shell env or `frontend/.env.local`. Root `.env` not auto-loaded by Next.

### interfaces
- consumes: all backend endpoints (see cross-repo contracts).

## cross-repo

### contracts

- name: GET /health
  type: api-endpoint
  producer: backend
  consumer: frontend (smoke only)
  shape:
  ```
  Response 200 { status: "ok", version: string }
  ```

- name: GET /api/conversations
  type: api-endpoint
  producer: backend
  consumer: frontend
  shape:
  ```
  Response 200 [{ id: uuid, title: string, created_at: iso8601, updated_at: iso8601 }]
  ```

- name: POST /api/conversations
  type: api-endpoint
  producer: backend
  consumer: frontend
  shape:
  ```
  Request  { title: string }   // 1..=200 chars
  Response 201 { id: uuid, title: string, created_at: iso8601, updated_at: iso8601 }
  Errors   400 VALIDATION
  ```

- name: GET /api/conversations/:id/messages
  type: api-endpoint
  producer: backend
  consumer: frontend
  shape:
  ```
  Response 200 [{ id: uuid, conversation_id: uuid, role: "user"|"assistant"|"system", content: string, created_at: iso8601 }]
  Errors   404 NOT_FOUND
  ```

- name: POST /api/conversations/:id/messages
  type: api-endpoint
  producer: backend
  consumer: frontend
  shape:
  ```
  Request  { content: string }   // 1..=32000 chars
  Response 201 {
    user_message: Message,
    assistant_message: Message
  }
  Errors   400 VALIDATION, 404 NOT_FOUND, 502 UPSTREAM
  Behavior backend persists user msg → calls OpenRouter chat (non-stream) → persists assistant msg → returns both.
  ```

- name: POST /api/conversations/:id/stream
  type: api-endpoint (SSE)
  producer: backend
  consumer: frontend
  shape:
  ```
  Request  { content: string }
  Response 200 text/event-stream:
    event: token        data: <text chunk>
    event: token        data: <text chunk>
    ...
    event: done         data: { "message_id": uuid }
    event: error        data: { "code": str, "message": str }   // on failure
  Behavior backend persists user msg first → opens OpenRouter stream → relays delta tokens → on completion persists assistant msg → emits done. On client disconnect → persist partial assistant msg.
  ```

- name: ErrorBody
  type: shared-shape
  shape:
  ```
  { error: { code: "VALIDATION"|"NOT_FOUND"|"UPSTREAM"|"INTERNAL", message: string } }
  ```

### dependencies
- backend must expose all endpoints + start successfully → frontend can develop against real API.
  reason: frontend Playwright smoke mocks routes; dev experience needs live backend.
- root `.env` must exist w/ `OPENROUTER_API_KEY` set locally → backend boots. `.env.example` already at root.
  reason: backend `Config::from_env` fails fast w/o key.
- CORS `FRONTEND_ORIGIN` must match frontend dev server origin → browser allows requests.
  reason: default `http://localhost:3000` matches Next.js dev port.

### risks
- risk: SSE through CORS w/ credentials misconfig.
  impact: stream blocked in browser.
  mitigation: CORS layer allows `FRONTEND_ORIGIN`, expose `content-type`, omit credentials. Frontend uses `fetch(..., { credentials: 'omit' })`.
- risk: OpenRouter free model returns 429 / interrupted stream.
  impact: chat fails silently.
  mitigation: backend maps 429 → `Upstream` 502 w/ message. SSE emits `event: error`. Frontend ErrorBanner shows + retry.
- risk: SQLite write contention under streaming + concurrent req.
  impact: locked DB errors.
  mitigation: single `SqlitePool` w/ `max_connections=5`, `journal_mode=WAL` pragma on connect.
- risk: Secret leak via tracked `.env`.
  impact: provider key exposure.
  mitigation: `.gitignore` already excludes `.env*` except `.env.example`. CI/review verifies no key in tracked files.

## Implementation Strategy

1. Backend scaffold: Cargo.toml + main.rs hello-world + /health → boots.
2. Backend DB: migration + sqlx pool + models.
3. Backend conversations endpoints + tests.
4. Backend OpenRouter trait + Mock + HTTP impl + messages send (non-stream) + tests.
5. Backend SSE stream endpoint + tests w/ mock.
6. Backend CORS + tracing + final wiring.
7. Frontend scaffold: Next + TS + Tailwind + base layout.
8. Frontend lib (zod + api client) + hooks (useConversations, useMessages, useCreateConversation).
9. Frontend components: Sidebar, ChatWindow, MessageBubble, Composer, Markdown.
10. Frontend streaming hook + integrate into ChatWindow.
11. Frontend Vitest component tests.
12. Frontend Playwright smoke.
13. Root README rewrite + verify all commands.

## Execution Order
1. backend (full impl + tests) → 2. frontend (full impl + tests).

## Conventions

### backend
- snake_case files + fns, PascalCase types.
- Errors via `Result<T, AppError>`. No `unwrap()` outside tests/main bootstrap.
- `?` propagation. `From<sqlx::Error> for AppError` etc.
- Handlers: `async fn(State(s): State<AppState>, Path(id): Path<Uuid>, Json(body): Json<DTO>) -> Result<Json<Resp>, AppError>`.
- Tests: integration style under `tests/`, share helper `fn test_app() -> Router` via local mod.
- `tracing::info!`/`warn!`/`error!` w/ structured fields. No `println!`.

### frontend
- Components PascalCase `.tsx`. Hooks camelCase `use*`. Files match export name.
- Tailwind utility-first. Avoid custom CSS beyond `globals.css`.
- Server components default; mark `'use client'` only when state/effects/event handlers needed (Sidebar, ChatWindow, Composer, hooks consumers).
- All API responses parsed via zod. Never `as` cast.
- Imports: `@/components/...`, `@/hooks/...`, `@/lib/...`.
- Test files colocated under `__tests__/` mirroring component name.

### root
- README sections: Overview, Prerequisites, Setup, Env, Run Backend, Run Frontend, Tests, Troubleshooting.
- Exact commands documented:
  - backend tests: `cd backend && cargo test`
  - backend run: `cd backend && cargo run --bin server`
  - frontend install: `cd frontend && npm install`
  - frontend dev: `cd frontend && npm run dev`
  - frontend tests: `cd frontend && npm test`
  - frontend e2e: `cd frontend && npm run test:e2e:install && npm run test:e2e`

ARCPLAN_COMPLETE
