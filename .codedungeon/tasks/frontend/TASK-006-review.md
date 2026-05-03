# Review TASK-006 — Frontend API schemas + hooks

PROJECT_RULES_STATUS: approved
PROJECT_RULES_DIGEST: 54eae19e90a1b403912baee95e0294f1ff7bdf2aff6afb8d07af8d0744677dfe
PROJECT_RULES_READ: yes

Verdict: APPROVED

## Acceptance check
- schemas.ts → Conversation, Message, SendMessageResponse, ErrorBody, Health all present + zod-typed → PASS.
- types.ts → infers from zod, never `as` cast → PASS.
- api.ts → API_BASE = `process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://localhost:8080"` → PASS. ApiError thrown for structured backend errors + zod parse failure (parse() throws ZodError → propagates) → PASS.
- useConversations queryKey `["conversations"]` stable → PASS.
- useMessages queryKey `["messages", conversationId]` stable → PASS.
- useStreamMessage AbortController on each send + reset → PASS. On `event:done` invalidates `messages` + `conversations` queries → PASS.

## Guardrails
- All API via apiFetch wrapper → no raw fetch in components → PASS.
- Stream hook fetch direct (acceptable — SSE not JSON; bypasses zod intentionally) → noted.
- Zero `any` types → PASS.
- No OpenRouter calls from frontend → PASS.

## Minor
- useStreamMessage on disconnect/abort does not persist partial buffer client-side → backend discards the interrupted in-flight turn (per arcplan). Acceptable.

REVIEW_COMPLETE: TASK-006
