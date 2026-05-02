# Review TASK-009 — Vitest component tests

PROJECT_RULES_STATUS: approved
PROJECT_RULES_DIGEST: 54eae19e90a1b403912baee95e0294f1ff7bdf2aff6afb8d07af8d0744677dfe
PROJECT_RULES_READ: yes

Verdict: APPROVED

## Acceptance check
- MessageBubble.test → user right-align + plain text, assistant markdown bold → `<strong>`, streaming cursor span present → PASS (3 tests).
- Composer.test → Enter sends, Shift+Enter newline + no send, disabled blocks → PASS (3 tests).
- Sidebar.test → seeded list renders both, active conv carries aria-current="page", New chat button visible → PASS (2 tests).
- jsdom env via vitest.config.ts → PASS.
- vitest.setup.ts imports jest-dom matchers → PASS.

## Test quality
- Fresh QueryClient seeded per test via `withQuery` helper → no cache leakage.
- `vi.mock("@/lib/api")` blocks real fetch in Sidebar test → safe.
- Accessible queries (`getByLabelText`, `getByText`) preferred over class selectors → PASS.

## Coverage
- 8/8 tests green via `npm test`.

REVIEW_COMPLETE: TASK-009
