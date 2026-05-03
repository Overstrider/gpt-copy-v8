# Review TASK-008 — Chat UI

PROJECT_RULES_STATUS: approved
PROJECT_RULES_DIGEST: 54eae19e90a1b403912baee95e0294f1ff7bdf2aff6afb8d07af8d0744677dfe
PROJECT_RULES_READ: yes

Verdict: APPROVED

## Acceptance check
- Sidebar lists conversations, highlights active w/ `aria-current="page"`, "New chat" → useCreateConversation → onSelect(new id) → PASS.
- ChatWindow → EmptyState when no `conversationId`; MessageList + Composer when active → PASS.
- Composer Enter submits, Shift+Enter newline, disabled blocks, `aria-label="Message"` + `aria-label="Send"` → PASS.
- Mobile sidebar hidden < md, drawer toggled via Menu button (`md:hidden`) → PASS.
- Active conversation in URL `?c=<id>` via useSearchParams + router.replace → PASS.

## Server/Client boundaries
- ChatShell, Sidebar, ConversationItem, ChatWindow, MessageList, MessageBubble, Markdown, Composer, ErrorBanner all `"use client"` → correct (state/events/effects).
- EmptyState, LoadingDots no `"use client"` → correct (Server-safe, no hooks).

## Markdown safety
- Markdown.tsx uses react-markdown + remark-gfm + `skipHtml` → PASS. No raw HTML injection sink used.

## a11y
- Buttons + textarea labeled, aria-current on active conv, role="alert" on ErrorBanner → PASS.

## Notes
- ChatWindow useEffect deps lint disabled for `conversationId`-only reset → acceptable, intentional.
- EmptyState condition simplified to `!conversationId` (vs original `&& messages.data?.length !== undefined`) → cleaner, behaviorally equivalent.

REVIEW_COMPLETE: TASK-008
