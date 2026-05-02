"use client";
import { useEffect, useRef } from "react";
import type { Message } from "@/lib/types";
import { MessageBubble } from "./MessageBubble";

export interface MessageListProps {
  messages: Message[];
  streamingText: string | null;
}

export function MessageList({ messages, streamingText }: MessageListProps) {
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
