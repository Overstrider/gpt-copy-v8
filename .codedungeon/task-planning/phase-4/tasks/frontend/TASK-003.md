# TASK-003: Frontend foundation and toolchain

PROJECT_RULES_STATUS: approved
PROJECT_RULES_DIGEST: 54eae19e90a1b403912baee95e0294f1ff7bdf2aff6afb8d07af8d0744677dfe
PROJECT_RULES_READ: yes

Repo: frontend
Kind: dev
Wave: 2
Parallel Group: frontend-foundation
Owner Role: frontend
Depends On: TASK-001

## Objective
Create the Next.js 15 App Router TypeScript app scaffold with Tailwind, providers, Vitest, and Playwright configuration.

## Context
- Use frontendplan.md for exact package scripts, dependencies, config files, and App Router structure.
- The first screen should be the usable chat shell, not a marketing page.
- OpenRouter keys must not appear in frontend code or env variables.

## Write Scope
- frontend/package.json
- frontend/tsconfig.json
- frontend/next.config.ts
- frontend/tailwind.config.ts
- frontend/postcss.config.mjs
- frontend/.eslintrc.json
- frontend/vitest.config.ts
- frontend/vitest.setup.ts
- frontend/playwright.config.ts
- frontend/src/app/globals.css
- frontend/src/app/layout.tsx
- frontend/src/app/providers.tsx
- frontend/src/app/page.tsx

## Acceptance Criteria
- frontend/package.json includes scripts for dev, build, lint, test, test:e2e, and test:e2e:install.
- TypeScript is strict and uses @/* path alias to frontend/src/*.
- Tailwind globals load successfully and the App Router layout mounts Providers.
- Playwright webServer starts npm run dev on port 3000.

## Verification Commands
- npm --prefix frontend install
- npm --prefix frontend run lint

## Risk Notes
- Next.js 15 lint config may require adjustment if next lint is unavailable or deprecated.
- Do not create a landing page; page.tsx should host the chat application shell.

