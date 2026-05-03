import { z } from "zod";
import {
  ConversationSchema,
  MessageSchema,
  RoleSchema,
  SendMessageResponseSchema,
  ErrorBodySchema,
} from "./schemas";

export type Conversation = z.infer<typeof ConversationSchema>;
export type Message = z.infer<typeof MessageSchema>;
export type Role = z.infer<typeof RoleSchema>;
export type SendMessageResponse = z.infer<typeof SendMessageResponseSchema>;
export type ErrorBody = z.infer<typeof ErrorBodySchema>;
