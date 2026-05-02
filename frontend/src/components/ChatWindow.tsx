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

export function ChatWindow({ conversationId, onConversationCreated }: ChatWindowProps) {
  const messages = useMessages(conversationId);
  const stream = useStreamMessage();
  const create = useCreateConversation();

  useEffect(() => {
    stream.reset();
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

  return (
    <section className="flex flex-1 flex-col">
      {messages.isError && <ErrorBanner message={messages.error?.message ?? "Failed to load."} />}
      {stream.error && <ErrorBanner message={stream.error} />}
      <div className="flex-1 overflow-y-auto">
        {!conversationId ? (
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
