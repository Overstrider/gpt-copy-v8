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
            let evt: string | null = null;
            let data = "";
            for (const line of raw.split("\n")) {
              if (line.startsWith("event:")) evt = line.slice(6).trim();
              else if (line.startsWith("data:")) {
                // Per SSE spec: strip a single optional leading space after `data:`,
                // preserve all other whitespace (trailing spaces are significant tokens).
                const rest = line.slice(5);
                data += rest.startsWith(" ") ? rest.slice(1) : rest;
              }
            }
            if (evt === "token") setBuffer((b) => b + data);
            else if (evt === "done") {
              setStatus("done");
              qc.invalidateQueries({ queryKey: ["messages", conversationId] });
              qc.invalidateQueries({ queryKey: ["conversations"] });
            } else if (evt === "error") {
              setStatus("error");
              setError(parseStreamError(data));
            } else if (evt && process.env.NODE_ENV !== "production") {
              console.warn(`Unhandled SSE event: ${evt}`);
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

function parseStreamError(data: string): string {
  if (!data) return "stream error";
  try {
    const parsed = JSON.parse(data) as { message?: unknown };
    if (typeof parsed.message === "string" && parsed.message.trim()) {
      return parsed.message;
    }
  } catch {
    // Non-JSON SSE errors are already human-readable.
  }
  return data;
}
