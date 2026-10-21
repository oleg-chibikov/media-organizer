import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { Pane } from "./Pane";

afterEach(cleanup);

describe("Pane", () => {
  it("closes on Escape and returns focus to its button", () => {
    render(
      <Pane
        renderTrigger={({ open, toggle }) => (
          <button type="button" aria-expanded={open} onClick={toggle}>
            Menu
          </button>
        )}
      >
        <p>Panel</p>
      </Pane>
    );
    const trigger = screen.getByRole("button", { name: "Menu" });

    fireEvent.click(trigger);
    expect(trigger).toHaveAttribute("aria-expanded", "true");
    fireEvent.keyDown(document, { key: "Escape" });

    expect(screen.queryByText("Panel")).not.toBeInTheDocument();
    expect(trigger).toHaveAttribute("aria-expanded", "false");
    expect(trigger).toHaveFocus();
  });
});
