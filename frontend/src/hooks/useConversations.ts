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
