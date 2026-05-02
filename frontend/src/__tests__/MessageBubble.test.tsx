import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { MessageBubble } from "@/components/MessageBubble";

describe("MessageBubble", () => {
  it("renders user role right-aligned with plain text", () => {
    render(<MessageBubble role="user" content="hello" />);
    const wrap = screen.getByTestId("bubble-user");
    expect(wrap.className).toMatch(/justify-end/);
    expect(screen.getByText("hello")).toBeInTheDocument();
  });

  it("renders assistant markdown bold as <strong>", () => {
    render(<MessageBubble role="assistant" content="**bold**" />);
    const wrap = screen.getByTestId("bubble-assistant");
    expect(wrap.className).toMatch(/justify-start/);
    expect(wrap.querySelector("strong")?.textContent).toBe("bold");
  });

  it("shows pulsing cursor when streaming", () => {
    render(<MessageBubble role="assistant" content="" streaming />);
    const wrap = screen.getByTestId("bubble-assistant");
    expect(wrap.querySelector("span.animate-pulse")).not.toBeNull();
  });
});
