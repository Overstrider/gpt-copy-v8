import { z } from "zod";

export const RoleSchema = z.enum(["user", "assistant", "system"]);
export type Role = z.infer<typeof RoleSchema>;

export const ConversationSchema = z.object({
  id: z.string().uuid(),
  title: z.string().min(1).max(200),
  created_at: z.string(),
  updated_at: z.string(),
});
export const ConversationListSchema = z.array(ConversationSchema);

export const MessageSchema = z.object({
  id: z.string().uuid(),
  conversation_id: z.string().uuid(),
  role: RoleSchema,
  content: z.string(),
  created_at: z.string(),
});
export const MessageListSchema = z.array(MessageSchema);

export const SendMessageResponseSchema = z.object({
  user_message: MessageSchema,
  assistant_message: MessageSchema,
});

export const ErrorBodySchema = z.object({
  error: z.object({
    code: z.enum(["VALIDATION", "NOT_FOUND", "UPSTREAM", "INTERNAL"]),
    message: z.string(),
  }),
});

export const HealthSchema = z.object({ status: z.literal("ok"), version: z.string() });
