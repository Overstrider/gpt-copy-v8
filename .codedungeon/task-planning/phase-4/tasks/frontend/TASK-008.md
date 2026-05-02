# TASK-008: Chat application UI

PROJECT_RULES_STATUS: approved
PROJECT_RULES_DIGEST: 54eae19e90a1b403912baee95e0294f1ff7bdf2aff6afb8d07af8d0744677dfe
PROJECT_RULES_READ: yes

Repo: frontend
Kind: dev
Wave: 4
Parallel Group: frontend-ui
Owner Role: frontend
Depends On: TASK-006

## Objective
Build the ChatGPT-style interface with sidebar conversations, transcript, markdown assistant bubbles, composer, loading/error states, and mobile drawer behavior.

## Context
- Use lucide-react icons for common actions.
- Assistant markdown should render through react-markdown and remark-gfm without raw HTML.
- Active conversation state is stored in the ?c=<id> URL query parameter.

## Write Scope
- frontend/src/components/ChatShell.tsx
- frontend/src/components/Sidebar.tsx
- frontend/src/components/ConversationItem.tsx
- frontend/src/components/ChatWindow.tsx
- frontend/src/components/MessageList.tsx
- frontend/src/components/MessageBubble.tsx
- frontend/src/components/Markdown.tsx
- frontend/src/components/Composer.tsx
- frontend/src/components/EmptyState.tsx
- frontend/src/components/ErrorBanner.tsx
- frontend/src/components/LoadingDots.tsx
- frontend/src/app/page.tsx
- frontend/src/app/globals.css

## Acceptance Criteria
- Sidebar lists conversations, highlights active conversation with aria-current, and can create a new chat.
- ChatWindow shows EmptyState without active conversation and MessageList plus Composer when active.
- Composer supports Enter to send, Shift+Enter for newline, disabled state while streaming, and accessible labels Message and Send.
- Mobile width hides sidebar by default and exposes a menu-driven drawer without horizontal overflow.

## Verification Commands
- npm --prefix frontend run lint
- npm --prefix frontend run build

## Risk Notes
- Do not add explanatory feature text in the UI; build the usable app directly.
- Keep text inside buttons and bubbles responsive without overlap.

