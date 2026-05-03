import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { Sidebar } from "@/components/Sidebar";
import type { Conversation } from "@/lib/types";

vi.mock("@/lib/api", () => ({
  API_BASE: "http://test",
  apiFetch: vi.fn().mockResolvedValue([]),
  ApiError: class extends Error {},
}));

function withQuery(ui: React.ReactNode, data: Conversation[]) {
  const qc = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  qc.setQueryData(["conversations"], data);
  return <QueryClientProvider client={qc}>{ui}</QueryClientProvider>;
}

const list: Conversation[] = [
  { id: "11111111-1111-1111-1111-111111111111", title: "First", created_at: "", updated_at: "" },
  { id: "22222222-2222-2222-2222-222222222222", title: "Second", created_at: "", updated_at: "" },
];

describe("Sidebar", () => {
  it("renders conversation items and highlights active one", () => {
    render(
      withQuery(
        <Sidebar
          activeId="22222222-2222-2222-2222-222222222222"
          onSelect={() => {}}
          drawerOpen={false}
          onToggleDrawer={() => {}}
        />,
        list,
      ),
    );
    expect(screen.getByText("First")).toBeInTheDocument();
    const active = screen.getByText("Second");
    expect(active.closest("button")?.getAttribute("aria-current")).toBe("page");
  });

  it("shows New chat button", () => {
    render(
      withQuery(
        <Sidebar
          activeId={null}
          onSelect={() => {}}
          drawerOpen={false}
          onToggleDrawer={() => {}}
        />,
        [],
      ),
    );
    expect(screen.getByText("New chat")).toBeInTheDocument();
  });
});
