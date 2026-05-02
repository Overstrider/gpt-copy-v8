# TASK-001: Root guardrails and environment placeholders

PROJECT_RULES_STATUS: approved
PROJECT_RULES_DIGEST: 54eae19e90a1b403912baee95e0294f1ff7bdf2aff6afb8d07af8d0744677dfe
PROJECT_RULES_READ: yes

Repo: root
Kind: dev
Wave: 1
Parallel Group: root-foundation
Owner Role: docs
Depends On: -

## Objective
Prepare monorepo-level hygiene so workers can add backend and frontend files without committing secrets or generated artifacts.

## Context
- The repo currently has README.md, CLAUDE.md, .env.example, prompts/full-v8.txt, and CodeDungeon planning files.
- Project rules require .env ignored, .env.example placeholder-only, and no generated databases, node_modules, build outputs, or test reports committed.

## Write Scope
- .gitignore
- .env.example
- README.md

## Acceptance Criteria
- .gitignore ignores .env, backend/*.sqlite, backend/*.sqlite-*, frontend/node_modules, frontend/.next, frontend/test-results, frontend/playwright-report, target directories, and other local build outputs.
- .env.example contains only placeholder values and defaults OPENROUTER_MODEL to nvidia/nemotron-3-super-120b-a12b:free.
- README keeps gpt-copy-v8 scope and has setup/run/test sections that can be finalized after implementation.

## Verification Commands
- git status --short
- Get-Content .env.example

## Risk Notes
- Do not add local .env contents or real OpenRouter keys.
- Keep README claims provisional until final verification tasks pass.

