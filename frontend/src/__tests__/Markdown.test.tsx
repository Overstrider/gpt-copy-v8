import { describe, it, expect } from "vitest";
import { render } from "@testing-library/react";
import { Markdown } from "@/components/Markdown";

describe("Markdown", () => {
  it("renders markdown bold as <strong>", () => {
    const { container } = render(<Markdown source="**bold**" />);
    expect(container.querySelector("strong")?.textContent).toBe("bold");
  });

  it("blocks remote images — no <img> emitted for ![alt](url)", () => {
    const { container } = render(
      <Markdown source="![pixel](https://attacker.example/pixel.png)" />,
    );
    expect(container.querySelector("img")).toBeNull();
    expect(container.textContent).toContain("[image: pixel]");
  });

  it("does not render raw HTML script tags (skipHtml)", () => {
    const { container } = render(
      <Markdown source={"plaintext line\n\n<script>window.__pwn=1</script>"} />,
    );
    expect(container.querySelector("script")).toBeNull();
    expect(container.textContent).toContain("plaintext line");
  });
});
