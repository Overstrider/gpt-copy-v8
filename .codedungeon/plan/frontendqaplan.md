# qaplan: frontend

## meta
- repo: frontend
- lang: TypeScript 5
- project_mode: BOOTSTRAP
- PROJECT_RULES_STATUS: approved
- PROJECT_RULES_DIGEST: 54eae19e90a1b403912baee95e0294f1ff7bdf2aff6afb8d07af8d0744677dfe
- PROJECT_RULES_READ: yes

## test-strategy
- lang: TypeScript
- test-types: integration, e2e
- integration-framework: Vitest 2 + @testing-library/react 16 + jsdom 25
- api-test-approach: N/A → frontend repo. Backend HTTP coverage owned by backend qaplan.
- e2e-framework: Playwright 1.48 (chromium only, route mocking — no real backend)
- existing-tests-dir: none → BOOTSTRAP
- existing-patterns: none → follow patterns prescribed by frontendplan.md and crystal-ball-e2e skill
- test-runner-command:
  - unit: `npm run test` → `vitest run`
  - e2e: `npm run test:e2e:install && npm run test:e2e` → `playwright test`

## definition-of-done

### Feature: Next.js 15 App Router shell
- [ ] `npm install` succeeds → `frontend/node_modules/` populated
- [ ] `npm run lint` exits 0 → no eslint errors
- [ ] `npm run build` exits 0 → `.next/` produced w/ no TS errors
- [ ] `npm run dev` boots → `GET http://localhost:3000/` returns 200 w/ HTML containing root layout
- [ ] `<html lang="en">` shell + `<Providers>` mounted → React Query devtools-free runtime, no console errors

### Feature: zod schemas + apiFetch wrapper
- [ ] `src/lib/schemas.ts` exports ConversationSchema, MessageSchema, SendMessageResponseSchema, ErrorBodySchema, HealthSchema
- [ ] `apiFetch` validates every res via zod → throws `ApiError` on parse fail
- [ ] `apiFetch` reads `NEXT_PUBLIC_API_BASE_URL` env, defaults `http://localhost:8080`
- [ ] No `as` casts in lib/api.ts or hooks → enforce via grep

### Feature: TanStack Query hooks
- [ ] `useConversations` queryKey `["conversations"]`, staleTime 5000
- [ ] `useMessages(id)` enabled iff id non-null, queryKey `["messages", id]`
- [ ] `useCreateConversation` invalidates `["conversations"]` on success
- [ ] `useStreamMessage` aborts in-flight req on new send → AbortController reused

### Feature: SSE streaming
- [ ] `useStreamMessage.send` parses `event: token` + `event: done` frames split by `\n\n`
- [ ] Token frames append `data:` payload to buffer state
- [ ] `done` frame → status `done` + invalidates `["messages", id]` and `["conversations"]`
- [ ] `error` frame → status `error` + sets error string
- [ ] Component unmount during stream → AbortController fires, no setState-after-unmount warnings

### Feature: ChatShell + URL state
- [ ] Active conversation id stored in `?c=<uuid>` search param
- [ ] Selecting conversation → `router.replace(/?c=<id>)` → URL updates without full reload
- [ ] Reload at `/?c=<id>` → app re-mounts w/ same active conversation

### Feature: Sidebar
- [ ] Renders list from `useConversations`
- [ ] Active item carries `aria-current="page"`
- [ ] "New chat" button → calls `useCreateConversation.mutateAsync` → on success calls `onSelect(newId)`
- [ ] Mobile (<768px): drawer hidden by default, Menu button toggles `drawerOpen`
- [ ] Loading state shows "Loading…" text while query pending
- [ ] Error state shows red "Failed to load conversations." text on query error

### Feature: ChatWindow + MessageList
- [ ] No active conversation → renders `<EmptyState>` w/ copy "Start a new chat from the sidebar."
- [ ] Active conversation → renders MessageList from `useMessages.data`
- [ ] Streaming → appends synthetic assistant bubble w/ `streamingText` content + pulsing cursor
- [ ] Auto-scrolls to bottom on messages or streamingText change
- [ ] `messages.isError` → renders `<ErrorBanner>` above list

### Feature: MessageBubble
- [ ] role=user → `data-testid="bubble-user"`, `justify-end`, `bg-zinc-700 text-zinc-50`, plain text (no markdown)
- [ ] role=assistant → `data-testid="bubble-assistant"`, `justify-start`, `bg-zinc-100 text-zinc-900`, markdown via `react-markdown` + `remark-gfm`
- [ ] Assistant `**bold**` → renders `<strong>bold</strong>`
- [ ] `streaming` prop → cursor span w/ `animate-pulse` class present

### Feature: Composer
- [ ] Enter key (no Shift) → calls `onSend(trimmed)` w/ textarea value, then clears textarea
- [ ] Shift+Enter → inserts newline, does NOT call `onSend`
- [ ] `disabled` prop → blocks submit (Enter no-op, Send button disabled)
- [ ] Empty/whitespace value → Send button disabled, Enter no-op
- [ ] Send button has `aria-label="Send"` + textarea has `aria-label="Message"`

### Feature: E2E smoke (mocked backend)
- [ ] User opens `/` → empty state visible
- [ ] User types "hello" in Composer + clicks Send → POST /api/conversations mock fires
- [ ] After create → URL contains `?c=<CONV_ID>` AND assistant bubble appears
- [ ] Assistant bubble contains concatenated streamed text "Hello world"
- [ ] No browser console errors during flow

## integration-tests

### test-module: MessageBubble
- what: render branches per role + streaming flag
- location: `frontend/src/__tests__/MessageBubble.test.tsx`
- cases:
  - user role → `getByTestId("bubble-user")` className matches `/justify-end/`. `getByText("hello")` present. No `<strong>` even if content contains `**`.
  - assistant role + content `**bold**` → `getByTestId("bubble-assistant")` className matches `/justify-start/`. `wrap.querySelector("strong")?.textContent === "bold"`.
  - assistant + streaming=true + content="" → `wrap.querySelector("span.animate-pulse")` non-null.
- references: none → first test in repo, follow Vitest + RTL patterns from frontendplan.md
- run-command: `npm run test -- src/__tests__/MessageBubble.test.tsx`

### test-module: Composer
- what: keyboard handling + disabled gating
- location: `frontend/src/__tests__/Composer.test.tsx`
- cases:
  - user types "hi" → presses Enter → `onSend` called once w/ "hi"
  - user types "a" → Shift+Enter → types "b" → `onSend` NOT called. Textarea value contains `\n`.
  - `disabled` prop set → user types "x" + Enter → `onSend` NOT called.
  - empty textarea → Send button disabled (`aria-disabled` or `disabled` attr) → Enter no-op.
- references: `@testing-library/user-event` 14 patterns (`userEvent.setup()`, `user.keyboard("{Shift>}{Enter}{/Shift}")`)
- run-command: `npm run test -- src/__tests__/Composer.test.tsx`

### test-module: Sidebar
- what: list render + active highlight
- location: `frontend/src/__tests__/Sidebar.test.tsx`
- cases:
  - seed QueryClient cache via `qc.setQueryData(["conversations"], list)` → render Sidebar w/ `activeId=<second.id>` → `getByText("First")` present, `getByText("Second").closest("button")?.getAttribute("aria-current") === "page"`.
  - empty list → no items rendered, "New chat" button visible.
  - mock `@/lib/api` → `apiFetch` resolves `[]` so internal queries don't crash during paint.
- references: TanStack Query test pattern → wrap in `<QueryClientProvider>` w/ pre-seeded cache
- run-command: `npm run test -- src/__tests__/Sidebar.test.tsx`

### test-runner-config
- vitest.config.ts → environment `jsdom`, globals true, setupFiles `./vitest.setup.ts`, alias `@` → `./src`
- vitest.setup.ts → `import "@testing-library/jest-dom/vitest"` → matchers attached
- include glob: `src/**/*.test.{ts,tsx}`

## api-tests
skip → frontend repo. No HTTP endpoints owned here. Backend curl coverage owned by `backendqaplan.md`.

## e2e-tests

### playwright-config
- testDir: `./e2e`
- baseURL: `http://localhost:3000`
- fullyParallel: false (single smoke spec, single worker safe)
- retries: 0 (CI may bump to 2)
- reporter: list (default; add html on CI)
- trace: on-first-retry
- screenshot: only-on-failure
- video: retain-on-failure
- projects: chromium only (Desktop Chrome device preset)
- webServer: `npm run dev`, url `http://localhost:3000`, reuseExistingServer iff `!CI`, timeout 120000
- auth: NOT required → no `## Test Auth` in CLAUDE.md, no real backend, `page.route()` stubs all `/api/**`

### test-flow: send-message-streaming-smoke
- file: `frontend/e2e/send-message.spec.ts`
- prerequisite: dev server up at `http://localhost:3000`. No DB. No backend. All `/api/**` mocked via `page.route()`.
- page-object: skip → single smoke spec, POM overhead unjustified for one flow
- mocks (set BEFORE `page.goto`):
  1. `**/api/conversations` GET → empty `[]` until first POST. After POST, returns `[{id: CONV_ID, title: "hello", created_at, updated_at}]`. Use closure flag `createdOnce`.
  2. `**/api/conversations` POST → 201 w/ JSON body `{id: CONV_ID, title: "hello", created_at: "2026-05-02T00:00:00Z", updated_at: "2026-05-02T00:00:00Z"}`. Sets `createdOnce = true`.
  3. `**/api/conversations/${CONV_ID}/messages` GET → 200 w/ body `"[]"`.
  4. `**/api/conversations/${CONV_ID}/stream` POST → 200, `content-type: text/event-stream`, body `"event: token\ndata: Hello \n\nevent: token\ndata: world\n\nevent: done\ndata: {\"message_id\":\"<ASSISTANT_ID>\"}\n\n"`.
- steps:
  1. Navigate `page.goto("/")`. Assert `getByText("Start a new chat from the sidebar.")` visible.
  2. Locate Composer textarea via `page.getByLabel("Message")`. Action: `fill("hello")`.
  3. Locate Send button via `page.getByLabel("Send")`. Action: `click()`.
  4. Assert `expect(page.getByTestId("bubble-assistant")).toBeVisible()` → assistant bubble paints.
  5. Assert `expect(page.getByTestId("bubble-assistant")).toContainText("Hello world")` → streamed tokens concatenated correctly.
  6. Assert `expect(page).toHaveURL(/\?c=33333333-3333-3333-3333-333333333333/)` → ChatShell wrote active conv to URL.
- selectors (priority per crystal-ball-e2e):
  1. `getByLabel("Message")` → textarea (P2 in skill)
  2. `getByLabel("Send")` → button (P2 via aria-label)
  3. `getByTestId("bubble-assistant")` → P5 fallback. Justified: bubble has no semantic role, content varies (markdown).
  4. `getByText("Start a new chat from the sidebar.")` → P4 for empty-state copy assertion
- assertions: web-first, auto-retrying. `toBeVisible`, `toContainText`, `toHaveURL`.
- anti-patterns-to-avoid:
  - NO `page.waitForTimeout(...)` → use auto-waiting assertions
  - NO `page.locator("div.flex >> nth=2")` → use semantic locators
  - NO shared state across tests → spec is single-test, but if extended, isolate via `beforeEach`
  - NO real network calls → every `**/api/**` MUST be intercepted via `page.route` BEFORE `page.goto`
  - NO `expect(...).toBeTruthy()` on Locator → use `toBeVisible`
- console-error guard: register `page.on("pageerror", e => { throw e })` and `page.on("console", m => { if (m.type() === "error") throw new Error(m.text()) })` at spec start → fail fast on runtime errors.

## frontend-ux-checks

### input-masks
skip → no masked fields in this feature (no phone, CPF, currency, date inputs). Composer textarea is free-text.

### form-validation-ux
- form: Composer
  submit-empty: empty/whitespace textarea → Send button disabled (visual `disabled:opacity-50`), Enter key no-op. No error toast (deliberate UX → silent block).
  error-visibility: stream error → `<ErrorBanner role="alert">` red bar above list, message text-red-200.
  fix-and-resubmit: after error, user types again + sends → ErrorBanner stays until next successful stream → `useStreamMessage.reset` clears on send start (verify: setError(null) in send fn).

### empty-loading-error-states
- component: Sidebar conversations list
  empty-state: list empty → no `<li>` items, "New chat" button still visible. No "no conversations" copy by spec → matches frontendplan.
  loading-state: `list.isLoading` → `<li class="text-xs text-zinc-500">Loading…</li>` shown.
  error-state: `list.isError` → `<li class="text-xs text-red-400">Failed to load conversations.</li>` shown.
- component: ChatWindow
  empty-state: no active conversation → `<EmptyState>` w/ copy "Start a new chat from the sidebar."
  loading-state: messages query loading → list area empty (no skeleton in spec) → acceptable for streaming app where loading is fast.
  error-state: `messages.isError` → `<ErrorBanner>` w/ `messages.error?.message`. `stream.error` → second `<ErrorBanner>` below.
- component: MessageList streaming bubble
  loading-state: `streamingText !== null` → synthetic assistant bubble w/ pulsing `▍` cursor span (`animate-pulse`).

### layout-integrity
- responsive-breakpoints: [375px, 768px, 1280px]
- desktop (>=768px / md): Sidebar visible (`md:flex`), 256px wide, ChatWindow flex-1.
- mobile (<768px): Sidebar hidden (`hidden`), Menu button (top-left, `md:hidden`) toggles drawer overlay.
- checks:
  - 1280px: sidebar + chat side-by-side, no horizontal scroll on body.
  - 768px: same layout, sidebar still pinned.
  - 375px: sidebar hidden by default, Menu button visible, ChatWindow fills width. Tap Menu → drawer slides in (`absolute inset-y-0 left-0 z-10 block`).
  - long messages: bubble caps at `max-w-[80%]`, `whitespace-pre-wrap` wraps, no overflow-x.
  - Composer pinned to bottom via flex column, textarea grows up to `max-h-48`.

## test-environment
- startup:
    method: local
    command: `cd frontend && npm install && npm run dev`
    base-url: `http://localhost:3000`
    health-check: `curl -sI http://localhost:3000/ | head -1` → `HTTP/1.1 200 OK`
    reference: frontendplan.md → `## verification` section
- seed-data: none → no DB. E2E uses `page.route()` mocks.
- auth: none → no auth in this feature. `## Test Auth` does NOT exist in CLAUDE.md and is NOT being created (TEST_AUTH_BEING_CREATED=false).
- env-vars:
  - `NEXT_PUBLIC_API_BASE_URL` (optional, defaults `http://localhost:8080`) → not exercised in E2E because all `/api/**` intercepted before fetch resolves URL host
- cleanup: none required → unit tests use fresh QueryClient per render. E2E single-test spec, no shared state.
- container: none → tests run on host Node 20+.
- prerequisites:
  - Node >= 20
  - npm
  - Playwright chromium browser → installed via `npm run test:e2e:install` (`playwright install --with-deps chromium`)
  - port 3000 free during E2E run

## risks
- vitest jsdom + react-markdown → `react-markdown` 9 ESM-only. Vitest config must NOT alias it to CJS. Risk: SSR/ESM mismatch → test imports fail. Mitigation: keep default Vitest module resolution; if fails, add `server.deps.inline: ["react-markdown", "remark-gfm"]` to vitest.config.
- Playwright `page.route` POST stream → `route.fulfill` w/ raw string body works for SSE iff `content-type: text/event-stream` set. Browser fetch will buffer entire body before exposing reader → streaming visual effect compressed but final assertion `toContainText("Hello world")` still passes.
- React 19 + @testing-library/react 16 → JSX runtime alignment. If `Cannot find namespace 'JSX'` errors surface in tests, add `import type {} from "react"` or use `ReactElement` return types instead of `JSX.Element`.

QAPLAN_COMPLETE: frontendqaplan.md
