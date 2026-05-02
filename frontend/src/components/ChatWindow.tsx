"use client";
import { useEffect, useRef } from "react";
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

export function ChatWindow({ conversationId, onConversationCreated }: ChatWindowProps) {
  const messages = useMessages(conversationId);
  const stream = useStreamMessage();
  const create = useCreateConversation();
  const lastConvRef = useRef<string | null>(conversationId);

  // Reset stream only when the user navigates between distinct existing conversations.
  // Skip the very first transition from null → newly-created id so the in-flight
  // stream that triggered the URL update is preserved through render.
  useEffect(() => {
    const prev = lastConvRef.current;
    if (prev !== null && prev !== conversationId) {
      stream.reset();
    }
    lastConvRef.current = conversationId;
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [conversationId]);

  const onSend = async (text: string) => {
    let id = conversationId;
    if (!id) {
      const c = await create.mutateAsync({ title: text.slice(0, 60) || "New chat" });
      id = c.id;
      onConversationCreated(id);
    }
    await stream.send(id, text);
  };

  // Keep streamed buffer visible while streaming OR after done until the refetched
  // messages list contains an assistant message (avoids flicker / disappearance).
  const list = messages.data ?? [];
  const hasAssistantPersisted = list.some((m) => m.role === "assistant");
  const showBuffer =
    stream.status === "streaming" ||
    (stream.status === "done" && stream.buffer.length > 0 && !hasAssistantPersisted);

  return (
    <section className="flex flex-1 flex-col">
      {messages.isError && <ErrorBanner message={messages.error?.message ?? "Failed to load."} />}
      {stream.error && <ErrorBanner message={stream.error} />}
      <div className="flex-1 overflow-y-auto">
        {!conversationId ? (
          <EmptyState />
        ) : (
          <MessageList
            messages={list}
            streamingText={showBuffer ? stream.buffer : null}
          />
        )}
      </div>
      <Composer disabled={stream.status === "streaming" || create.isPending} onSend={onSend} />
    </section>
  );
}
