# TASK-013: Final docs, secret scan, and handoff readiness

PROJECT_RULES_STATUS: approved
PROJECT_RULES_DIGEST: 54eae19e90a1b403912baee95e0294f1ff7bdf2aff6afb8d07af8d0744677dfe
PROJECT_RULES_READ: yes

Repo: root
Kind: fix
Wave: 7
Parallel Group: finalization
Owner Role: docs
Depends On: TASK-011, TASK-012

## Objective
Finalize README instructions, confirm ignored artifacts, scan tracked files for provider keys, and prepare the CodeDungeon completion handoff.

## Context
- Final CodeDungeon reports may claim COMPLETE only after verification is PASS and review is APPROVED.
- Project rules state workflows are PR-centered and final merge remains with the user.
- If project rules become stale only because requested source files were created, refresh and compact them before finalizing.

## Write Scope
- README.md
- .env.example
- .gitignore
- .codedungeon/project-rules.md
- .codedungeon/project-rules.compact.md

## Acceptance Criteria
- README includes setup, env, backend run, frontend run, backend tests, frontend tests, Playwright smoke, and troubleshooting.
- .env.example remains placeholder-only and contains OPENROUTER_API_KEY, OPENROUTER_MODEL, DATABASE_URL, BACKEND_HOST, BACKEND_PORT, FRONTEND_ORIGIN, and NEXT_PUBLIC_API_BASE_URL.
- Tracked-file scan finds no OpenRouter key material.
- Final handoff includes project rules status approved, digest 54eae19e90a1b403912baee95e0294f1ff7bdf2aff6afb8d07af8d0744677dfe, read yes.

## Verification Commands
- git status --short
- git ls-files | Select-String -Pattern "OPENROUTER_API_KEY|sk-or-|Bearer "
- Get-Content README.md
- Get-Content .env.example

## Risk Notes
- The simple grep can match placeholder names; reviewers must distinguish placeholder variable names from real key material.
- Do not mark final completion unless backend and frontend verification tasks both passed.

