import { z } from "zod";
import { ErrorBodySchema } from "./schemas";

export const API_BASE =
  process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://localhost:8080";

export class ApiError extends Error {
  constructor(
    public code: string,
    message: string,
    public status: number,
  ) {
    super(message);
    this.name = "ApiError";
  }
}

export async function apiFetch<T>(
  path: string,
  schema: z.ZodType<T>,
  init?: RequestInit,
): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, {
    credentials: "omit",
    headers: { "content-type": "application/json", ...(init?.headers ?? {}) },
    ...init,
  });
  if (!res.ok) {
    const body = await res.json().catch(() => null);
    const parsed = ErrorBodySchema.safeParse(body);
    if (parsed.success) {
      throw new ApiError(parsed.data.error.code, parsed.data.error.message, res.status);
    }
    throw new ApiError("INTERNAL", `HTTP ${res.status}`, res.status);
  }
  const json = await res.json();
  return schema.parse(json);
}

export function streamUrl(path: string): string {
  return `${API_BASE}${path}`;
}
