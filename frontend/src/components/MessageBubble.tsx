"use client";
import clsx from "clsx";
import type { Role } from "@/lib/types";
import { Markdown } from "./Markdown";

export interface MessageBubbleProps {
  role: Role;
  content: string;
  streaming?: boolean;
}

export function MessageBubble({ role, content, streaming }: MessageBubbleProps) {
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
