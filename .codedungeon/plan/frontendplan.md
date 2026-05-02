# frontendplan

## meta
- repo: frontend
- lang: TypeScript 5
- stack: Next.js 15 App Router, React 19, Tailwind 3, TanStack Query 5, zod 3, react-markdown 9, remark-gfm 4, lucide-react, clsx
- project_mode: BOOTSTRAP
- execution_order: shared lib (schemas/types/api) → hooks → components → app routes → tests
- PROJECT_RULES_STATUS: approved
- PROJECT_RULES_DIGEST: 54eae19e90a1b403912baee95e0294f1ff7bdf2aff6afb8d07af8d0744677dfe
- PROJECT_RULES_READ: yes

## scope
Bootstrap Next.js 15 App Router app under `frontend/`. Server-component shell + client islands for stateful UI. Sidebar lists conversations, ChatWindow renders transcript + Composer, MessageBubble renders markdown for assistant role. zod validates every backend response. TanStack Query caches lists/messages. Native fetch + ReadableStream parses SSE for token streaming. URL search param `?c=<id>` holds active conversation id (no Redux/Zustand). Vitest + Testing Library for component tests. One Playwright smoke spec stubs backend → asserts streamed assistant bubble.

## files

### frontend/package.json
action: create.

#### nextjs
Manifest. Type module. Scripts + deps below.

```jsonc
{
  "name": "gpt-copy-v8-frontend",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "next dev -p 3000",
    "build": "next build",
    "start": "next start -p 3000",
    "lint": "next lint",
    "test": "vitest run",
    "test:watch": "vitest",
    "test:e2e": "playwright test",
    "test:e2e:install": "playwright install --with-deps chromium"
  },
  "dependencies": {
    "next": "^15.0.0",
    "react": "^19.0.0",
    "react-dom": "^19.0.0",
    "@tanstack/react-query": "^5.0.0",
    "zod": "^3.23.0",
    "lucide-react": "^0.460.0",
    "react-markdown": "^9.0.0",
    "remark-gfm": "^4.0.0",
    "clsx": "^2.1.0"
  },
  "devDependencies": {
    "typescript": "^5.4.0",
    "@types/node": "^20.12.0",
    "@types/react": "^19.0.0",
    "@types/react-dom": "^19.0.0",
    "tailwindcss": "^3.4.0",
    "postcss": "^8.4.0",
    "autoprefixer": "^10.4.0",
    "vitest": "^2.1.0",
    "@testing-library/react": "^16.0.0",
    "@testing-library/jest-dom": "^6.5.0",
    "@testing-library/user-event": "^14.5.0",
    "jsdom": "^25.0.0",
    "@vitejs/plugin-react": "^4.3.0",
    "@playwright/test": "^1.48.0",
    "eslint": "^9.0.0",
    "eslint-config-next": "^15.0.0",
    "prettier": "^3.3.0"
  }
}
```

### frontend/tsconfig.json
action: create.

#### nextjs
Strict TS. Path alias `@/*` → `./src/*`. Next plugin enabled.

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "lib": ["dom", "dom.iterable", "esnext"],
    "allowJs": false,
    "skipLibCheck": true,
    "strict": true,
    "noEmit": true,
    "esModuleInterop": true,
    "module": "esnext",
    "moduleResolution": "bundler",
    "resolveJsonModule": true,
    "isolatedModules": true,
    "jsx": "preserve",
    "incremental": true,
    "plugins": [{ "name": "next" }],
    "baseUrl": ".",
    "paths": { "@/*": ["./src/*"] }
  },
  "include": ["next-env.d.ts", "**/*.ts", "**/*.tsx", ".next/types/**/*.ts"],
  "exclude": ["node_modules", "e2e"]
}
```

### frontend/next.config.ts
action: create.

#### nextjs
Minimal config. Strict mode on. No experimental flags.

```ts
import type { NextConfig } from "next";
const nextConfig: NextConfig = { reactStrictMode: true };
export default nextConfig;
```

### frontend/tailwind.config.ts
action: create.

#### nextjs
Scan `src/**/*.{ts,tsx}`. Neutral/zinc palette via defaults.

```ts
import type { Config } from "tailwindcss";
const config: Config = {
  content: ["./src/**/*.{ts,tsx}"],
  theme: { extend: {} },
  plugins: [],
};
export default config;
```

### frontend/postcss.config.mjs
action: create.

#### nextjs
Tailwind + autoprefixer.

```js
export default { plugins: { tailwindcss: {}, autoprefixer: {} } };
```

### frontend/.eslintrc.json
action: create.

#### nextjs
Extends `next/core-web-vitals`.

```json
{ "extends": ["next/core-web-vitals"] }
```

### frontend/vitest.config.ts
action: create.

#### nextjs
jsdom env, react plugin, setup file, alias `@/`.

```ts
import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";
import path from "node:path";
export default defineConfig({
  plugins: [react()],
  resolve: { alias: { "@": path.resolve(__dirname, "./src") } },
  test: {
    environment: "jsdom",
    globals: true,
    setupFiles: ["./vitest.setup.ts"],
    include: ["src/**/*.test.{ts,tsx}"],
    css: false,
  },
});
```

### frontend/vitest.setup.ts
action: create.

#### nextjs
Imports jest-dom matchers.

```ts
import "@testing-library/jest-dom/vitest";
```

### frontend/playwright.config.ts
action: create.

#### nextjs
webServer boots `npm run dev`. baseURL `http://localhost:3000`. testDir `./e2e`.

```ts
import { defineConfig, devices } from "@playwright/test";
export default defineConfig({
  testDir: "./e2e",
  fullyParallel: false,
  retries: 0,
  use: { baseURL: "http://localhost:3000", trace: "on-first-retry" },
  projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"] } }],
  webServer: {
    command: "npm run dev",
    url: "http://localhost:3000",
    reuseExistingServer: !process.env.CI,
    timeout: 120_000,
  },
});
```

### frontend/src/lib/schemas.ts
action: create.

#### nextjs
zod schemas. Source-of-truth for response shapes. Never `as` cast.

```ts
import { z } from "zod";

export const RoleSchema = z.enum(["user", "assistant", "system"]);
export type Role = z.infer<typeof RoleSchema>;

export const ConversationSchema = z.object({
  id: z.string().uuid(),
  title: z.string().min(1).max(200),
  created_at: z.string(),
  updated_at: z.string(),
});
export const ConversationListSchema = z.array(ConversationSchema);

export const MessageSchema = z.object({
  id: z.string().uuid(),
  conversation_id: z.string().uuid(),
  role: RoleSchema,
  content: z.string(),
  created_at: z.string(),
});
export const MessageListSchema = z.array(MessageSchema);

export const SendMessageResponseSchema = z.object({
  user_message: MessageSchema,
  assistant_message: MessageSchema,
});

export const ErrorBodySchema = z.object({
  error: z.object({
    code: z.enum(["VALIDATION", "NOT_FOUND", "UPSTREAM", "INTERNAL"]),
    message: z.string(),
  }),
});

export const HealthSchema = z.object({ status: z.literal("ok"), version: z.string() });
```

### frontend/src/lib/types.ts
action: create.

#### nextjs
Re-exports inferred types. Keeps consumers off zod direct.

```ts
import { z } from "zod";
import {
  ConversationSchema,
  MessageSchema,
  RoleSchema,
  SendMessageResponseSchema,
  ErrorBodySchema,
} from "./schemas";

export type Conversation = z.infer<typeof ConversationSchema>;
export type Message = z.infer<typeof MessageSchema>;
export type Role = z.infer<typeof RoleSchema>;
export type SendMessageResponse = z.infer<typeof SendMessageResponseSchema>;
export type ErrorBody = z.infer<typeof ErrorBodySchema>;
```

### frontend/src/lib/api.ts
action: create.

#### nextjs
Fetch wrapper. Validates via zod. Throws `ApiError`. Reads `NEXT_PUBLIC_API_BASE_URL`.

```ts
import { z } from "zod";
import { ErrorBodySchema } from "./schemas";

export const API_BASE =
  process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://localhost:8080";

export class ApiError extends Error {
  constructor(
    public code: string,
    message: string,
    public status: number,
  ) {
    super(message);
    this.name = "ApiError";
  }
}

export async function apiFetch<T>(
  path: string,
  schema: z.ZodType<T>,
  init?: RequestInit,
): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, {
    credentials: "omit",
    headers: { "content-type": "application/json", ...(init?.headers ?? {}) },
    ...init,
  });
  if (!res.ok) {
    const body = await res.json().catch(() => null);
    const parsed = ErrorBodySchema.safeParse(body);
    if (parsed.success) {
      throw new ApiError(parsed.data.error.code, parsed.data.error.message, res.status);
    }
    throw new ApiError("INTERNAL", `HTTP ${res.status}`, res.status);
  }
  const json = await res.json();
  return schema.parse(json);
}

export function streamUrl(path: string): string {
  return `${API_BASE}${path}`;
}
```

### frontend/src/hooks/useConversations.ts
action: create.

#### nextjs
Client hook. TanStack Query GET list. zod-validated.

```ts
"use client";
import { useQuery, type UseQueryResult } from "@tanstack/react-query";
import { apiFetch } from "@/lib/api";
import { ConversationListSchema } from "@/lib/schemas";
import type { Conversation } from "@/lib/types";

export function useConversations(): UseQueryResult<Conversation[], Error> {
  return useQuery({
    queryKey: ["conversations"],
    queryFn: () => apiFetch("/api/conversations", ConversationListSchema),
    staleTime: 5_000,
  });
}
```

### frontend/src/hooks/useMessages.ts
action: create.

#### nextjs
Client hook. Conditional enabled when id present.

```ts
"use client";
import { useQuery, type UseQueryResult } from "@tanstack/react-query";
import { apiFetch } from "@/lib/api";
import { MessageListSchema } from "@/lib/schemas";
import type { Message } from "@/lib/types";

export function useMessages(conversationId: string | null): UseQueryResult<Message[], Error> {
  return useQuery({
    queryKey: ["messages", conversationId],
    queryFn: () =>
      apiFetch(`/api/conversations/${conversationId}/messages`, MessageListSchema),
    enabled: !!conversationId,
    staleTime: 0,
  });
}
```

### frontend/src/hooks/useCreateConversation.ts
action: create.

#### nextjs
Client mutation. Invalidates `conversations`. Returns new id.

```ts
"use client";
import { useMutation, useQueryClient, type UseMutationResult } from "@tanstack/react-query";
import { apiFetch } from "@/lib/api";
import { ConversationSchema } from "@/lib/schemas";
import type { Conversation } from "@/lib/types";

export interface CreateConversationInput { title: string }

export function useCreateConversation(): UseMutationResult<
  Conversation, Error, CreateConversationInput
> {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (input) =>
      apiFetch("/api/conversations", ConversationSchema, {
        method: "POST",
        body: JSON.stringify(input),
      }),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["conversations"] }),
  });
}
```

### frontend/src/hooks/useStreamMessage.ts
action: create.

#### nextjs
Native fetch + ReadableStream reader. SSE line parse: split `\n\n` events, then per-event split lines, trim `event:` + `data:` prefixes. Dispatch token append. Abort on unmount via AbortController. On `event: done` invalidate `messages`.

```ts
"use client";
import { useCallback, useRef, useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { API_BASE } from "@/lib/api";

export type StreamStatus = "idle" | "streaming" | "done" | "error";

export interface UseStreamMessageResult {
  status: StreamStatus;
  buffer: string;
  error: string | null;
  send: (conversationId: string, content: string) => Promise<void>;
  reset: () => void;
}

export function useStreamMessage(): UseStreamMessageResult {
  const [status, setStatus] = useState<StreamStatus>("idle");
  const [buffer, setBuffer] = useState("");
  const [error, setError] = useState<string | null>(null);
  const ctrlRef = useRef<AbortController | null>(null);
  const qc = useQueryClient();

  const reset = useCallback(() => {
    ctrlRef.current?.abort();
    ctrlRef.current = null;
    setStatus("idle");
    setBuffer("");
    setError(null);
  }, []);

  const send = useCallback(
    async (conversationId: string, content: string) => {
      ctrlRef.current?.abort();
      const ctrl = new AbortController();
      ctrlRef.current = ctrl;
      setBuffer("");
      setError(null);
      setStatus("streaming");
      try {
        const res = await fetch(
          `${API_BASE}/api/conversations/${conversationId}/stream`,
          {
            method: "POST",
            credentials: "omit",
            headers: { "content-type": "application/json" },
            body: JSON.stringify({ content }),
            signal: ctrl.signal,
          },
        );
        if (!res.ok || !res.body) throw new Error(`HTTP ${res.status}`);
        const reader = res.body.getReader();
        const decoder = new TextDecoder();
        let pending = "";
        while (true) {
          const { value, done } = await reader.read();
          if (done) break;
          pending += decoder.decode(value, { stream: true });
          let idx: number;
          while ((idx = pending.indexOf("\n\n")) !== -1) {
            const raw = pending.slice(0, idx);
            pending = pending.slice(idx + 2);
            let evt = "message";
            let data = "";
            for (const line of raw.split("\n")) {
              if (line.startsWith("event:")) evt = line.slice(6).trim();
              else if (line.startsWith("data:")) data += line.slice(5).trim();
            }
            if (evt === "token") setBuffer((b) => b + data);
            else if (evt === "done") {
              setStatus("done");
              qc.invalidateQueries({ queryKey: ["messages", conversationId] });
              qc.invalidateQueries({ queryKey: ["conversations"] });
            } else if (evt === "error") {
              setStatus("error");
              setError(data || "stream error");
            }
          }
        }
      } catch (e) {
        if ((e as Error).name === "AbortError") return;
        setStatus("error");
        setError((e as Error).message);
      }
    },
    [qc],
  );

  return { status, buffer, error, send, reset };
}
```

### frontend/src/app/globals.css
action: create.

#### nextjs
Tailwind directives + base. Sets full-height html/body for flex layout.

```css
@tailwind base;
@tailwind components;
@tailwind utilities;

html, body, #__next { height: 100%; }
body { @apply bg-zinc-950 text-zinc-100 antialiased; }
```

### frontend/src/app/providers.tsx
action: create.

#### nextjs
Client component. Single QueryClient instance per mount. Wraps children.

```tsx
"use client";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useState, type ReactNode } from "react";

export interface ProvidersProps { children: ReactNode }

export function Providers({ children }: ProvidersProps): JSX.Element {
  const [client] = useState(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: { retry: 1, refetchOnWindowFocus: false },
        },
      }),
  );
  return <QueryClientProvider client={client}>{children}</QueryClientProvider>;
}
```

### frontend/src/app/layout.tsx
action: create.

#### nextjs
Server Component. HTML shell. Wraps children in `<Providers>`. Imports globals.css.

```tsx
import type { Metadata } from "next";
import "./globals.css";
import { Providers } from "./providers";

export const metadata: Metadata = {
  title: "GPT Copy v8",
  description: "ChatGPT-style chat over OpenRouter.",
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" className="h-full">
      <body className="h-full">
        <Providers>{children}</Providers>
      </body>
    </html>
  );
}
```

### frontend/src/app/page.tsx
action: create.

#### nextjs
Server Component shell. Renders client `<ChatShell/>` (which reads `useSearchParams`). Layout: full-viewport flex row, sidebar 260px (`w-64 hidden md:block`), main `flex-1`. Mobile drawer toggled inside Sidebar.

```tsx
import { ChatShell } from "@/components/ChatShell";

export default function Page() {
  return (
    <main className="flex h-full w-full flex-row overflow-hidden">
      <ChatShell />
    </main>
  );
}
```

### frontend/src/components/ChatShell.tsx
action: create.

#### nextjs
Client Component. Reads `?c=<id>` via `useSearchParams` + `useRouter`. Owns mobile drawer open state. Renders Sidebar + ChatWindow.

```tsx
"use client";
import { useSearchParams, useRouter } from "next/navigation";
import { useCallback, useState } from "react";
import { Sidebar } from "./Sidebar";
import { ChatWindow } from "./ChatWindow";

export function ChatShell(): JSX.Element {
  const params = useSearchParams();
  const router = useRouter();
  const activeId = params.get("c");
  const [drawerOpen, setDrawerOpen] = useState(false);

  const setActive = useCallback(
    (id: string | null) => {
      const next = new URLSearchParams(params.toString());
      if (id) next.set("c", id);
      else next.delete("c");
      router.replace(`/?${next.toString()}`);
      setDrawerOpen(false);
    },
    [params, router],
  );

  return (
    <>
      <Sidebar
        activeId={activeId}
        onSelect={setActive}
        drawerOpen={drawerOpen}
        onToggleDrawer={() => setDrawerOpen((v) => !v)}
      />
      <ChatWindow conversationId={activeId} onConversationCreated={setActive} />
    </>
  );
}
```

### frontend/src/components/Sidebar.tsx
action: create.

#### nextjs
Client. Lists conversations via `useConversations`. "New chat" calls `useCreateConversation` → on success picks new id. Mobile drawer hidden by default; toggled via Menu icon button visible `< md`.

```tsx
"use client";
import clsx from "clsx";
import { Plus, Menu, Loader } from "lucide-react";
import { useConversations } from "@/hooks/useConversations";
import { useCreateConversation } from "@/hooks/useCreateConversation";
import { ConversationItem } from "./ConversationItem";

export interface SidebarProps {
  activeId: string | null;
  onSelect: (id: string | null) => void;
  drawerOpen: boolean;
  onToggleDrawer: () => void;
}

export function Sidebar(props: SidebarProps): JSX.Element {
  const { activeId, onSelect, drawerOpen, onToggleDrawer } = props;
  const list = useConversations();
  const create = useCreateConversation();

  const onNewChat = async () => {
    const c = await create.mutateAsync({ title: "New chat" });
    onSelect(c.id);
  };

  return (
    <>
      <button
        type="button"
        aria-label="Toggle sidebar"
        className="md:hidden absolute left-3 top-3 z-20 rounded p-2 bg-zinc-800"
        onClick={onToggleDrawer}
      >
        <Menu size={18} />
      </button>
      <aside
        data-testid="sidebar"
        className={clsx(
          "w-64 shrink-0 border-r border-zinc-800 bg-zinc-900 p-3 flex flex-col gap-3",
          "md:block",
          drawerOpen ? "absolute inset-y-0 left-0 z-10 block" : "hidden md:flex",
        )}
      >
        <button
          type="button"
          onClick={onNewChat}
          disabled={create.isPending}
          className="flex items-center gap-2 rounded bg-zinc-800 px-3 py-2 text-sm hover:bg-zinc-700 disabled:opacity-50"
        >
          {create.isPending ? <Loader size={16} className="animate-spin" /> : <Plus size={16} />}
          New chat
        </button>
        <ul className="flex-1 overflow-y-auto space-y-1">
          {list.data?.map((c) => (
            <ConversationItem
              key={c.id}
              conversation={c}
              active={c.id === activeId}
              onSelect={onSelect}
            />
          ))}
          {list.isLoading && <li className="text-xs text-zinc-500">Loading…</li>}
          {list.isError && (
            <li className="text-xs text-red-400">Failed to load conversations.</li>
          )}
        </ul>
      </aside>
    </>
  );
}
```

### frontend/src/components/ConversationItem.tsx
action: create.

#### nextjs
Client. Highlight when active.

```tsx
"use client";
import clsx from "clsx";
import type { Conversation } from "@/lib/types";

export interface ConversationItemProps {
  conversation: Conversation;
  active: boolean;
  onSelect: (id: string) => void;
}

export function ConversationItem({ conversation, active, onSelect }: ConversationItemProps): JSX.Element {
  return (
    <li>
      <button
        type="button"
        aria-current={active ? "page" : undefined}
        onClick={() => onSelect(conversation.id)}
        className={clsx(
          "w-full truncate rounded px-2 py-1.5 text-left text-sm",
          active ? "bg-zinc-700 text-white" : "text-zinc-300 hover:bg-zinc-800",
        )}
      >
        {conversation.title}
      </button>
    </li>
  );
}
```

### frontend/src/components/ChatWindow.tsx
action: create.

#### nextjs
Client. Reads messages via `useMessages`. Owns `useStreamMessage`. Empty state when no conversationId. Auto-creates on first send if needed (passed via `onConversationCreated`). Renders MessageList + Composer + ErrorBanner.

```tsx
"use client";
import { useEffect } from "react";
import { useMessages } from "@/hooks/useMessages";
import { useStreamMessage } from "@/hooks/useStreamMessage";
import { useCreateConversation } from "@/hooks/useCreateConversation";
import { MessageList } from "./MessageList";
import { Composer } from "./Composer";
import { EmptyState } from "./EmptyState";
import { ErrorBanner } from "./ErrorBanner";

export interface ChatWindowProps {
  conversationId: string | null;
  onConversationCreated: (id: string) => void;
}

export function ChatWindow({ conversationId, onConversationCreated }: ChatWindowProps): JSX.Element {
  const messages = useMessages(conversationId);
  const stream = useStreamMessage();
  const create = useCreateConversation();

  useEffect(() => stream.reset, [conversationId]);

  const onSend = async (text: string) => {
    let id = conversationId;
    if (!id) {
      const c = await create.mutateAsync({ title: text.slice(0, 60) || "New chat" });
      id = c.id;
      onConversationCreated(id);
    }
    await stream.send(id, text);
  };

  return (
    <section className="flex flex-1 flex-col">
      {messages.isError && <ErrorBanner message={messages.error?.message ?? "Failed to load."} />}
      {stream.error && <ErrorBanner message={stream.error} />}
      <div className="flex-1 overflow-y-auto">
        {!conversationId && messages.data?.length !== undefined ? (
          <EmptyState />
        ) : (
          <MessageList
            messages={messages.data ?? []}
            streamingText={stream.status === "streaming" ? stream.buffer : null}
          />
        )}
      </div>
      <Composer disabled={stream.status === "streaming" || create.isPending} onSend={onSend} />
    </section>
  );
}
```

### frontend/src/components/MessageList.tsx
action: create.

#### nextjs
Client. Maps messages → MessageBubble. Auto-scrolls to bottom on change via ref + effect. Streaming bubble appended when `streamingText` non-null.

```tsx
"use client";
import { useEffect, useRef } from "react";
import type { Message } from "@/lib/types";
import { MessageBubble } from "./MessageBubble";

export interface MessageListProps {
  messages: Message[];
  streamingText: string | null;
}

export function MessageList({ messages, streamingText }: MessageListProps): JSX.Element {
  const ref = useRef<HTMLDivElement>(null);
  useEffect(() => {
    ref.current?.scrollTo({ top: ref.current.scrollHeight });
  }, [messages, streamingText]);
  return (
    <div ref={ref} className="mx-auto flex max-w-3xl flex-col gap-3 px-4 py-6">
      {messages.map((m) => (
        <MessageBubble key={m.id} role={m.role} content={m.content} />
      ))}
      {streamingText !== null && (
        <MessageBubble role="assistant" content={streamingText} streaming />
      )}
    </div>
  );
}
```

### frontend/src/components/MessageBubble.tsx
action: create.

#### nextjs
Client. User → right-aligned, `bg-zinc-700`, plain text. Assistant → left-aligned, `bg-zinc-100 text-zinc-900`, renders via `<Markdown>`. Streaming variant shows pulsing cursor span.

```tsx
"use client";
import clsx from "clsx";
import type { Role } from "@/lib/types";
import { Markdown } from "./Markdown";

export interface MessageBubbleProps {
  role: Role;
  content: string;
  streaming?: boolean;
}

export function MessageBubble({ role, content, streaming }: MessageBubbleProps): JSX.Element {
  const isUser = role === "user";
  return (
    <div
      data-testid={`bubble-${role}`}
      className={clsx("flex w-full", isUser ? "justify-end" : "justify-start")}
    >
      <div
        className={clsx(
          "max-w-[80%] rounded-2xl px-4 py-2 text-sm leading-relaxed whitespace-pre-wrap",
          isUser ? "bg-zinc-700 text-zinc-50" : "bg-zinc-100 text-zinc-900",
        )}
      >
        {isUser ? (
          content
        ) : (
          <Markdown source={content} />
        )}
        {streaming && <span aria-hidden className="ml-1 inline-block w-2 animate-pulse">▍</span>}
      </div>
    </div>
  );
}
```

### frontend/src/components/Markdown.tsx
action: create.

#### nextjs
Client. Wraps `react-markdown` w/ `remark-gfm`. `skipHtml` default → no raw HTML. Locks to safe primitives.

```tsx
"use client";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";

export interface MarkdownProps { source: string }

export function Markdown({ source }: MarkdownProps): JSX.Element {
  return (
    <div className="prose prose-sm prose-zinc max-w-none">
      <ReactMarkdown remarkPlugins={[remarkGfm]} skipHtml>
        {source}
      </ReactMarkdown>
    </div>
  );
}
```

### frontend/src/components/Composer.tsx
action: create.

#### nextjs
Client. Controlled textarea. Enter → submit; Shift+Enter → newline. Disabled prop blocks input + Send. Send icon from lucide. Auto-grow up to 8 rows.

```tsx
"use client";
import { useState, useCallback, type KeyboardEvent } from "react";
import { Send } from "lucide-react";
import clsx from "clsx";

export interface ComposerProps {
  disabled?: boolean;
  onSend: (text: string) => void | Promise<void>;
}

export function Composer({ disabled, onSend }: ComposerProps): JSX.Element {
  const [value, setValue] = useState("");

  const submit = useCallback(async () => {
    const text = value.trim();
    if (!text || disabled) return;
    setValue("");
    await onSend(text);
  }, [value, disabled, onSend]);

  const onKeyDown = (e: KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      void submit();
    }
  };

  return (
    <form
      data-testid="composer"
      onSubmit={(e) => {
        e.preventDefault();
        void submit();
      }}
      className="border-t border-zinc-800 bg-zinc-900 p-3"
    >
      <div className="mx-auto flex max-w-3xl items-end gap-2">
        <textarea
          aria-label="Message"
          value={value}
          onChange={(e) => setValue(e.target.value)}
          onKeyDown={onKeyDown}
          rows={1}
          disabled={disabled}
          placeholder="Send a message…"
          className="min-h-[44px] max-h-48 flex-1 resize-y rounded bg-zinc-800 px-3 py-2 text-sm outline-none focus:ring-1 focus:ring-zinc-500 disabled:opacity-50"
        />
        <button
          type="submit"
          disabled={disabled || value.trim().length === 0}
          className={clsx(
            "rounded bg-zinc-700 px-3 py-2 text-sm hover:bg-zinc-600 disabled:opacity-50",
          )}
          aria-label="Send"
        >
          <Send size={16} />
        </button>
      </div>
    </form>
  );
}
```

### frontend/src/components/EmptyState.tsx
action: create.

#### nextjs
Server-safe (no hooks). Centered placeholder.

```tsx
export function EmptyState(): JSX.Element {
  return (
    <div className="flex h-full items-center justify-center text-zinc-400">
      <p className="text-sm">Start a new chat from the sidebar.</p>
    </div>
  );
}
```

### frontend/src/components/ErrorBanner.tsx
action: create.

#### nextjs
Client. Red bar w/ AlertCircle icon. Optional `onRetry`.

```tsx
"use client";
import { AlertCircle } from "lucide-react";

export interface ErrorBannerProps { message: string; onRetry?: () => void }

export function ErrorBanner({ message, onRetry }: ErrorBannerProps): JSX.Element {
  return (
    <div role="alert" className="flex items-center gap-2 bg-red-900/40 px-4 py-2 text-sm text-red-200">
      <AlertCircle size={16} />
      <span className="flex-1">{message}</span>
      {onRetry && (
        <button type="button" onClick={onRetry} className="underline">
          Retry
        </button>
      )}
    </div>
  );
}
```

### frontend/src/components/LoadingDots.tsx
action: create.

#### nextjs
Server-safe. Three pulsing dots.

```tsx
export function LoadingDots(): JSX.Element {
  return (
    <span aria-label="Loading" className="inline-flex gap-1">
      <span className="h-1.5 w-1.5 animate-pulse rounded-full bg-zinc-400 [animation-delay:0ms]" />
      <span className="h-1.5 w-1.5 animate-pulse rounded-full bg-zinc-400 [animation-delay:150ms]" />
      <span className="h-1.5 w-1.5 animate-pulse rounded-full bg-zinc-400 [animation-delay:300ms]" />
    </span>
  );
}
```

### frontend/src/__tests__/MessageBubble.test.tsx
action: create.

#### nextjs
Vitest + Testing Library. Asserts: user → right alignment + plain text; assistant → markdown bold rendered as `<strong>`; streaming → cursor span present.

```tsx
import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { MessageBubble } from "@/components/MessageBubble";

describe("MessageBubble", () => {
  it("renders user role right-aligned with plain text", () => {
    render(<MessageBubble role="user" content="hello" />);
    const wrap = screen.getByTestId("bubble-user");
    expect(wrap.className).toMatch(/justify-end/);
    expect(screen.getByText("hello")).toBeInTheDocument();
  });

  it("renders assistant markdown bold as <strong>", () => {
    render(<MessageBubble role="assistant" content="**bold**" />);
    const wrap = screen.getByTestId("bubble-assistant");
    expect(wrap.className).toMatch(/justify-start/);
    expect(wrap.querySelector("strong")?.textContent).toBe("bold");
  });

  it("shows pulsing cursor when streaming", () => {
    render(<MessageBubble role="assistant" content="" streaming />);
    const wrap = screen.getByTestId("bubble-assistant");
    expect(wrap.querySelector("span.animate-pulse")).not.toBeNull();
  });
});
```

### frontend/src/__tests__/Composer.test.tsx
action: create.

#### nextjs
Asserts: Enter triggers `onSend`; Shift+Enter inserts newline (no send); disabled blocks send.

```tsx
import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Composer } from "@/components/Composer";

describe("Composer", () => {
  it("submits on Enter", async () => {
    const onSend = vi.fn();
    const user = userEvent.setup();
    render(<Composer onSend={onSend} />);
    const ta = screen.getByLabelText("Message");
    await user.type(ta, "hi");
    await user.keyboard("{Enter}");
    expect(onSend).toHaveBeenCalledWith("hi");
  });

  it("Shift+Enter inserts newline, does not send", async () => {
    const onSend = vi.fn();
    const user = userEvent.setup();
    render(<Composer onSend={onSend} />);
    const ta = screen.getByLabelText("Message") as HTMLTextAreaElement;
    await user.type(ta, "a");
    await user.keyboard("{Shift>}{Enter}{/Shift}");
    await user.type(ta, "b");
    expect(onSend).not.toHaveBeenCalled();
    expect(ta.value).toContain("\n");
  });

  it("does not send when disabled", async () => {
    const onSend = vi.fn();
    const user = userEvent.setup();
    render(<Composer onSend={onSend} disabled />);
    const ta = screen.getByLabelText("Message");
    await user.type(ta, "x");
    await user.keyboard("{Enter}");
    expect(onSend).not.toHaveBeenCalled();
  });
});
```

### frontend/src/__tests__/Sidebar.test.tsx
action: create.

#### nextjs
Wraps Sidebar in QueryClientProvider w/ seeded cache → asserts items rendered + active item has `aria-current="page"`. Mocks `useCreateConversation` via real provider but disables fetch. Uses `setQueryData` to seed.

```tsx
import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { Sidebar } from "@/components/Sidebar";
import type { Conversation } from "@/lib/types";

vi.mock("@/lib/api", () => ({
  API_BASE: "http://test",
  apiFetch: vi.fn().mockResolvedValue([]),
  ApiError: class extends Error {},
}));

function withQuery(ui: React.ReactNode, data: Conversation[]) {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  qc.setQueryData(["conversations"], data);
  return <QueryClientProvider client={qc}>{ui}</QueryClientProvider>;
}

const list: Conversation[] = [
  { id: "11111111-1111-1111-1111-111111111111", title: "First", created_at: "", updated_at: "" },
  { id: "22222222-2222-2222-2222-222222222222", title: "Second", created_at: "", updated_at: "" },
];

describe("Sidebar", () => {
  it("renders conversation items and highlights active one", () => {
    render(
      withQuery(
        <Sidebar
          activeId="22222222-2222-2222-2222-222222222222"
          onSelect={() => {}}
          drawerOpen={false}
          onToggleDrawer={() => {}}
        />,
        list,
      ),
    );
    expect(screen.getByText("First")).toBeInTheDocument();
    const active = screen.getByText("Second");
    expect(active.closest("button")?.getAttribute("aria-current")).toBe("page");
  });
});
```

### frontend/e2e/send-message.spec.ts
action: create.

#### nextjs
Playwright smoke. Stubs all `**/api/**`. List GET → empty array, then post-create GET → one entry. POST conversations → 201 w/ id. POST stream → SSE response w/ token + done events. Asserts assistant bubble appears w/ streamed text.

```ts
import { test, expect } from "@playwright/test";

const CONV_ID = "33333333-3333-3333-3333-333333333333";
const ASSISTANT_ID = "44444444-4444-4444-4444-444444444444";

test("send message renders streamed assistant bubble", async ({ page }) => {
  let createdOnce = false;

  await page.route("**/api/conversations", async (route) => {
    if (route.request().method() === "GET") {
      const body = createdOnce
        ? [{ id: CONV_ID, title: "hello", created_at: "2026-05-02T00:00:00Z", updated_at: "2026-05-02T00:00:00Z" }]
        : [];
      await route.fulfill({ status: 200, contentType: "application/json", body: JSON.stringify(body) });
      return;
    }
    if (route.request().method() === "POST") {
      createdOnce = true;
      await route.fulfill({
        status: 201,
        contentType: "application/json",
        body: JSON.stringify({
          id: CONV_ID, title: "hello",
          created_at: "2026-05-02T00:00:00Z", updated_at: "2026-05-02T00:00:00Z",
        }),
      });
      return;
    }
    await route.continue();
  });

  await page.route(`**/api/conversations/${CONV_ID}/messages`, async (route) => {
    await route.fulfill({ status: 200, contentType: "application/json", body: "[]" });
  });

  await page.route(`**/api/conversations/${CONV_ID}/stream`, async (route) => {
    const sse =
      "event: token\ndata: Hello \n\n" +
      "event: token\ndata: world\n\n" +
      `event: done\ndata: {"message_id":"${ASSISTANT_ID}"}\n\n`;
    await route.fulfill({
      status: 200,
      headers: { "content-type": "text/event-stream", "cache-control": "no-cache" },
      body: sse,
    });
  });

  await page.goto("/");
  await page.getByLabel("Message").fill("hello");
  await page.getByLabel("Send").click();

  await expect(page.getByTestId("bubble-assistant")).toBeVisible();
  await expect(page.getByTestId("bubble-assistant")).toContainText("Hello world");
});
```

## test-strategy

### unit (Vitest + Testing Library, jsdom)
- `MessageBubble.test.tsx` → user vs assistant alignment, markdown `<strong>` rendering, streaming cursor span.
- `Composer.test.tsx` → Enter submits, Shift+Enter newline, disabled blocks submission.
- `Sidebar.test.tsx` → renders seeded list, active item carries `aria-current="page"`.
- Setup: `vitest.setup.ts` imports `@testing-library/jest-dom/vitest` so matchers attach.

### e2e (Playwright)
- `e2e/send-message.spec.ts` → mocks `/api/**` via `page.route`. Covers list GET (empty), POST create, messages GET, POST stream emitting `token` + `done` SSE events. Asserts `bubble-assistant` appears w/ concatenated streamed text.
- `webServer` boots `npm run dev` on port 3000. baseURL pinned. `reuseExistingServer` outside CI.

### coverage targets
- Every component listed in `## affected-modules` either has a direct test or is covered transitively via `Sidebar`/`Composer`/`MessageBubble` + the smoke spec.
- Hooks exercised via Sidebar test (TanStack Query cache seeding) + Playwright (full streaming path).

## verification
Run from `frontend/`:
- `npm install`
- `npm run lint`
- `npm run test`
- `npm run build`
- `npm run test:e2e:install && npm run test:e2e` (smoke)

Pass criteria: lint clean, all Vitest specs green, `next build` succeeds w/o type errors, Playwright smoke green.

PLAN_COMPLETE: frontendplan.md
