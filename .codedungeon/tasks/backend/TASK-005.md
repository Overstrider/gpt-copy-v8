# TASK-005: Conversation REST routes

PROJECT_RULES_STATUS: approved
PROJECT_RULES_DIGEST: 54eae19e90a1b403912baee95e0294f1ff7bdf2aff6afb8d07af8d0744677dfe
PROJECT_RULES_READ: yes

Repo: backend
Kind: dev
Wave: 3
Parallel Group: backend-services
Owner Role: backend
Depends On: TASK-002

## Objective
Add list and create conversation endpoints with validation, ordering, timestamps, and persistence tests.

## Context
- Endpoints are GET /api/conversations and POST /api/conversations.
- Titles must be non-empty and at most 200 characters.
- List ordering is updated_at DESC.

## Write Scope
- backend/src/routes/conversations.rs
- backend/src/routes/mod.rs
- backend/src/models.rs
- backend/tests/conversations.rs

## Acceptance Criteria
- GET /api/conversations returns [] for an empty database.
- POST /api/conversations returns 201 with UUID id, title, created_at, and updated_at.
- Empty and too-long titles return 400 with error.code VALIDATION.
- Created conversations persist and appear in descending updated_at order.

## Verification Commands
- cargo test --manifest-path backend/Cargo.toml --test conversations

## Risk Notes
- Timestamp strings should be consistent enough for deterministic ordering assertions.
- Avoid DB query macros that require offline metadata.

