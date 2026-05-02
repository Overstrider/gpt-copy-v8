# TASK-009: Frontend component tests

PROJECT_RULES_STATUS: approved
PROJECT_RULES_DIGEST: 54eae19e90a1b403912baee95e0294f1ff7bdf2aff6afb8d07af8d0744677dfe
PROJECT_RULES_READ: yes

Repo: frontend
Kind: test
Wave: 5
Parallel Group: frontend-qa
Owner Role: qa
Depends On: TASK-008

## Objective
Add focused Vitest and Testing Library coverage for MessageBubble, Composer, and Sidebar behavior.

## Context
- frontendqaplan.md defines exact test modules and expected cases.
- Component tests should avoid real backend calls by seeding QueryClient or mocking the API wrapper.

## Write Scope
- frontend/src/__tests__/MessageBubble.test.tsx
- frontend/src/__tests__/Composer.test.tsx
- frontend/src/__tests__/Sidebar.test.tsx
- frontend/vitest.config.ts
- frontend/vitest.setup.ts

## Acceptance Criteria
- MessageBubble tests cover user alignment, assistant markdown strong rendering, and streaming cursor.
- Composer tests cover Enter send, Shift+Enter newline, disabled no-op, and empty-value disabled send.
- Sidebar tests cover seeded conversation list, active aria-current, empty list, and New chat visibility.
- All Vitest specs pass in jsdom.

## Verification Commands
- npm --prefix frontend run test

## Risk Notes
- TanStack Query tests must use a fresh QueryClient per test to prevent shared cache leakage.
- Use web-accessible queries where possible rather than brittle class-only selectors.

