# Review TASK-010 — Playwright mocked streaming smoke

PROJECT_RULES_STATUS: approved
PROJECT_RULES_DIGEST: 54eae19e90a1b403912baee95e0294f1ff7bdf2aff6afb8d07af8d0744677dfe
PROJECT_RULES_READ: yes

Verdict: APPROVED

## Acceptance check
- `page.route('**/api/conversations', …)` GET empty list pre-create, POST 201, GET list-with-one post-create → PASS.
- `page.route('**/api/conversations/:id/messages', …)` 200 [] → PASS.
- `page.route('**/api/conversations/:id/stream', …)` SSE token+token+done → PASS.
- All routes registered before `page.goto("/")` → PASS.
- Fills Message textarea, clicks Send → PASS.
- Asserts `bubble-assistant` visible + contains `Hello world` → PASS.
- Asserts URL contains `?c=<CONV_ID>` after creation → PASS.
- Fails on console errors / pageerror via captured array assertion → PASS.
- playwright.config webServer boots `npm run dev` port 3000, baseURL pinned, reuseExistingServer outside CI → PASS.

## Run gate
- `npx playwright test` executed: 1 passed (6.7s).

REVIEW_COMPLETE: TASK-010
