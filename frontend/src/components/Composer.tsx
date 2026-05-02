"use client";
import { useState, useCallback, type KeyboardEvent } from "react";
import { Send } from "lucide-react";
import clsx from "clsx";

export interface ComposerProps {
  disabled?: boolean;
  onSend: (text: string) => void | Promise<void>;
}

export function Composer({ disabled, onSend }: ComposerProps) {
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
