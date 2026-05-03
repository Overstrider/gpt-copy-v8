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
