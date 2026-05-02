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
