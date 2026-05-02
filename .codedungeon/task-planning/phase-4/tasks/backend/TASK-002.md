# TASK-002: Backend foundation and persistence

PROJECT_RULES_STATUS: approved
PROJECT_RULES_DIGEST: 54eae19e90a1b403912baee95e0294f1ff7bdf2aff6afb8d07af8d0744677dfe
PROJECT_RULES_READ: yes

Repo: backend
Kind: dev
Wave: 2
Parallel Group: backend-foundation
Owner Role: backend
Depends On: TASK-001

## Objective
Create the Rust 2024 Axum crate, configuration, shared state, error model, SQLite migration, database initializer, and health route.

## Context
- Use backendplan.md for exact dependency list and file responsibilities.
- Use sqlx runtime queries rather than query! macros, with sqlx::migrate! for migrations.
- Config defaults must match project rules and fail fast only on missing OPENROUTER_API_KEY at runtime startup.

## Write Scope
- backend/Cargo.toml
- backend/rust-toolchain.toml
- backend/migrations/0001_init.sql
- backend/src/lib.rs
- backend/src/main.rs
- backend/src/config.rs
- backend/src/error.rs
- backend/src/state.rs
- backend/src/db.rs
- backend/src/models.rs
- backend/src/routes/mod.rs
- backend/src/routes/health.rs
- backend/tests/common/mod.rs
- backend/tests/health.rs

## Acceptance Criteria
- backend/Cargo.toml declares edition = "2024" and crate name gpt_copy_v8_backend.
- Migration creates conversations and messages tables with FK cascade and message conversation/date index.
- GET /health returns JSON with status ok and version from CARGO_PKG_VERSION.
- AppError maps validation, not found, upstream, database, and internal errors to structured JSON error bodies.

## Verification Commands
- cargo fmt --manifest-path backend/Cargo.toml --check
- cargo test --manifest-path backend/Cargo.toml --test health

## Risk Notes
- sqlx migration path must work when commands are run with --manifest-path from repo root.
- Config tests should not require a real provider key.

