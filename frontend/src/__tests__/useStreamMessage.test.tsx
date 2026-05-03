import { describe, it, expect, vi, afterEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { useStreamMessage } from "@/hooks/useStreamMessage";

function withQuery(ui: React.ReactNode) {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return <QueryClientProvider client={qc}>{ui}</QueryClientProvider>;
}

function StreamHarness() {
  const stream = useStreamMessage();
  return (
    <>
      <button type="button" onClick={() => void stream.send("conv-1", "hello")}>
        send
      </button>
      <div data-testid="error">{stream.error}</div>
    </>
  );
}

function streamResponse(body: string) {
  return {
    ok: true,
    body: new ReadableStream({
      start(controller) {
        controller.enqueue(new TextEncoder().encode(body));
        controller.close();
      },
    }),
  } as Response;
}

describe("useStreamMessage", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("shows the message field from SSE error JSON", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn().mockResolvedValue(
        streamResponse(
          'event: error\ndata: {"code":"UPSTREAM","message":"too many requests"}\n\n',
        ),
      ),
    );

    render(withQuery(<StreamHarness />));
    await userEvent.click(screen.getByRole("button", { name: "send" }));

    await waitFor(() => {
      expect(screen.getByTestId("error").textContent).toBe("too many requests");
    });
  });
});
