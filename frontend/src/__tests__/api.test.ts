import { describe, expect, it, vi, afterEach } from "vitest";
import { apiFetch } from "@/lib/api";
import { ConversationListSchema } from "@/lib/schemas";

describe("apiFetch", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("preserves RATE_LIMITED error responses", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue({
        ok: false,
        status: 429,
        json: async () => ({
          error: {
            code: "RATE_LIMITED",
            message: "too many requests",
          },
        }),
      }),
    );

    await expect(apiFetch("/api/conversations", ConversationListSchema)).rejects.toMatchObject({
      code: "RATE_LIMITED",
      message: "too many requests",
      status: 429,
    });
  });
});
