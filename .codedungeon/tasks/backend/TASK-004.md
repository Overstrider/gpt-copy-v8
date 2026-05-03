# TASK-004: OpenRouter client abstraction

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
Implement the mockable OpenRouter client trait, HTTP client, streaming parser, error enum, and provider-focused tests.

## Context
- Project rules require OpenRouter calls to stay server-side and tests to avoid real provider calls.
- OpenRouter failures for 429, timeout, invalid response, and interrupted streams must not crash the backend.
- Handlers will depend on a trait object stored in AppState.

## Write Scope
- backend/src/openrouter.rs
- backend/src/error.rs
- backend/src/state.rs
- backend/tests/openrouter_mock.rs
- backend/tests/common/mod.rs

## Acceptance Criteria
- OpenRouterClient exposes chat and stream methods that accept model and message history.
- HttpOpenRouterClient sends bearer-authenticated requests to https://openrouter.ai/api/v1/chat/completions.
- Streaming parser emits content deltas, skips empty deltas, and terminates on data: [DONE].
- MockOpenRouterClient supports deterministic chat, deterministic token streams, and injectable failures.

## Verification Commands
- cargo test --manifest-path backend/Cargo.toml --test openrouter_mock

## Risk Notes
- Do not log Authorization headers or key material.
- Keep any configurable OpenRouter base URL test-only or non-secret.

