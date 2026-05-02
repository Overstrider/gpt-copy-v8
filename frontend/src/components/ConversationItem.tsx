"use client";
import clsx from "clsx";
import type { Conversation } from "@/lib/types";

export interface ConversationItemProps {
  conversation: Conversation;
  active: boolean;
  onSelect: (id: string) => void;
}

export function ConversationItem({ conversation, active, onSelect }: ConversationItemProps) {
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
