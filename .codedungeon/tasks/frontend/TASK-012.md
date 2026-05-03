# TASK-012: Frontend full verification

PROJECT_RULES_STATUS: approved
PROJECT_RULES_DIGEST: 54eae19e90a1b403912baee95e0294f1ff7bdf2aff6afb8d07af8d0744677dfe
PROJECT_RULES_READ: yes

Repo: frontend
Kind: test
Wave: 6
Parallel Group: verification
Owner Role: qa
Depends On: TASK-009, TASK-010

## Objective
Run and stabilize frontend install, lint, component tests, production build, and Playwright smoke after all frontend tasks land.

## Context
- This is the frontend verification gate required by project rules.
- Playwright remains mocked and must not require the backend server or OpenRouter.

## Write Scope
- frontend/**

## Acceptance Criteria
- npm install completes with package-lock.json generated if npm is the chosen package manager.
- npm run lint passes.
- npm run test passes.
- npm run build passes with no TypeScript errors.
- npm run test:e2e passes with mocked API routes.

## Verification Commands
- npm --prefix frontend install
- npm --prefix frontend run lint
- npm --prefix frontend run test
- npm --prefix frontend run build
- npm --prefix frontend run test:e2e:install
- npm --prefix frontend run test:e2e

## Risk Notes
- Do not commit frontend/node_modules, .next, test-results, or playwright-report.
- Keep Playwright smoke deterministic and independent from local backend availability.

