# TASK-010: Playwright mocked streaming smoke

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
Add one Playwright smoke test that sends a message through the UI and verifies streamed assistant text using mocked backend API routes.

## Context
- No real backend or OpenRouter call is required for this smoke test.
- Route mocks must be registered before page.goto.
- The test should assert empty state, sending, assistant bubble visibility, concatenated streamed text, and URL query state.

## Write Scope
- frontend/e2e/send-message.spec.ts
- frontend/playwright.config.ts

## Acceptance Criteria
- Playwright intercepts all /api/** routes before navigation.
- The smoke test fills the Message textarea, clicks Send, and sees an assistant bubble containing Hello world.
- The URL contains ?c=<mock-conversation-id> after conversation creation.
- The spec fails on page errors or browser console errors.

## Verification Commands
- npm --prefix frontend run test:e2e:install
- npm --prefix frontend run test:e2e

## Risk Notes
- Playwright browser install can be slow or unavailable in constrained environments; report that explicitly if blocked.
- Mock URL patterns must match NEXT_PUBLIC_API_BASE_URL default behavior.

