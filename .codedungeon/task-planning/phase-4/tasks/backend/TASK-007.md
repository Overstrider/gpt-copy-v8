# TASK-007: Message send and SSE routes

PROJECT_RULES_STATUS: approved
PROJECT_RULES_DIGEST: 54eae19e90a1b403912baee95e0294f1ff7bdf2aff6afb8d07af8d0744677dfe
PROJECT_RULES_READ: yes

Repo: backend
Kind: dev
Wave: 4
Parallel Group: backend-messages
Owner Role: backend
Depends On: TASK-004, TASK-005

## Objective
Add message listing, synchronous send, and streaming send endpoints with persistence, OpenRouter integration, validation, and tests.

## Context
- Depends on conversation persistence and OpenRouter client abstraction.
- Message content must be non-empty and at most 32000 characters.
- SSE should persist assistant output after completion and emit error events for upstream failures.

## Write Scope
- backend/src/routes/messages.rs
- backend/src/routes/mod.rs
- backend/src/models.rs
- backend/src/openrouter.rs
- backend/tests/messages.rs
- backend/tests/common/mod.rs

## Acceptance Criteria
- GET /api/conversations/:id/messages returns messages sorted created_at ASC and 404 for missing conversations.
- POST /api/conversations/:id/messages persists user and assistant messages and returns both with 201.
- POST /api/conversations/:id/stream emits token events, then done with assistant message id, and persists concatenated assistant content.
- Validation and upstream errors return structured JSON or SSE error events without panics.

## Verification Commands
- cargo test --manifest-path backend/Cargo.toml --test messages
- cargo test --manifest-path backend/Cargo.toml

## Risk Notes
- Persisting partial content on disconnect is subtle; at minimum ensure no crash and deterministic completed-stream persistence.
- Backpressure and stream lifetime must not hold mutable DB transactions across token emission.

