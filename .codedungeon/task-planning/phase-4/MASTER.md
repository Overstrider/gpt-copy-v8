# Task Planning Master

PROJECT_RULES_STATUS: approved
PROJECT_RULES_DIGEST: 54eae19e90a1b403912baee95e0294f1ff7bdf2aff6afb8d07af8d0744677dfe
PROJECT_RULES_READ: yes

| Wave | Task | Repo | Parallel Group | Depends On | Title |
|------|------|------|----------------|------------|-------|
| 1 | TASK-001 | root | root-foundation | - | Root guardrails and environment placeholders |
| 2 | TASK-002 | backend | backend-foundation | TASK-001 | Backend foundation and persistence |
| 2 | TASK-003 | frontend | frontend-foundation | TASK-001 | Frontend foundation and toolchain |
| 3 | TASK-004 | backend | backend-services | TASK-002 | OpenRouter client abstraction |
| 3 | TASK-005 | backend | backend-services | TASK-002 | Conversation REST routes |
| 3 | TASK-006 | frontend | frontend-data | TASK-003 | Frontend API schemas and hooks |
| 4 | TASK-007 | backend | backend-messages | TASK-004, TASK-005 | Message send and SSE routes |
| 4 | TASK-008 | frontend | frontend-ui | TASK-006 | Chat application UI |
| 5 | TASK-009 | frontend | frontend-qa | TASK-008 | Frontend component tests |
| 5 | TASK-010 | frontend | frontend-qa | TASK-008 | Playwright mocked streaming smoke |
| 6 | TASK-011 | backend | verification | TASK-007 | Backend full verification |
| 6 | TASK-012 | frontend | verification | TASK-009, TASK-010 | Frontend full verification |
| 7 | TASK-013 | root | finalization | TASK-011, TASK-012 | Final docs, secret scan, and handoff readiness |
