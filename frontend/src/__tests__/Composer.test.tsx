import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Composer } from "@/components/Composer";

describe("Composer", () => {
  it("submits on Enter", async () => {
    const onSend = vi.fn();
    const user = userEvent.setup();
    render(<Composer onSend={onSend} />);
    const ta = screen.getByLabelText("Message");
    await user.type(ta, "hi");
    await user.keyboard("{Enter}");
    expect(onSend).toHaveBeenCalledWith("hi");
  });

  it("Shift+Enter inserts newline, does not send", async () => {
    const onSend = vi.fn();
    const user = userEvent.setup();
    render(<Composer onSend={onSend} />);
    const ta = screen.getByLabelText("Message") as HTMLTextAreaElement;
    await user.type(ta, "a");
    await user.keyboard("{Shift>}{Enter}{/Shift}");
    await user.type(ta, "b");
    expect(onSend).not.toHaveBeenCalled();
    expect(ta.value).toContain("\n");
  });

  it("does not send when disabled", async () => {
    const onSend = vi.fn();
    const user = userEvent.setup();
    render(<Composer onSend={onSend} disabled />);
    const ta = screen.getByLabelText("Message");
    await user.type(ta, "x");
    await user.keyboard("{Enter}");
    expect(onSend).not.toHaveBeenCalled();
  });
});
