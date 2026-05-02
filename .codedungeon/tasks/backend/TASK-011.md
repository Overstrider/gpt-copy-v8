# TASK-011: Backend full verification

PROJECT_RULES_STATUS: approved
PROJECT_RULES_DIGEST: 54eae19e90a1b403912baee95e0294f1ff7bdf2aff6afb8d07af8d0744677dfe
PROJECT_RULES_READ: yes

Repo: backend
Kind: test
Wave: 6
Parallel Group: verification
Owner Role: qa
Depends On: TASK-007

## Objective
Run and stabilize backend formatting, linting, tests, and release build after all backend implementation tasks land.

## Context
- This is the backend verification gate required by project rules.
- Fix only backend-owned issues discovered by fmt, clippy, tests, or build.

## Write Scope
- backend/**

## Acceptance Criteria
- cargo fmt --check passes.
- cargo clippy --all-targets -- -D warnings passes.
- cargo test passes without external OpenRouter calls.
- cargo build --release succeeds.

## Verification Commands
- cargo fmt --manifest-path backend/Cargo.toml --check
- cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings
- cargo test --manifest-path backend/Cargo.toml
- cargo build --manifest-path backend/Cargo.toml --release

## Risk Notes
- Do not hide failures by weakening tests or removing required error handling.
- If clippy exposes design issues in shared types, coordinate with frontend contract expectations before changing JSON shapes.

