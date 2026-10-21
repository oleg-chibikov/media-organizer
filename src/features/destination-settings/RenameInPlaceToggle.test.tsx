import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { RenameInPlaceToggle } from "./RenameInPlaceToggle";

afterEach(cleanup);

describe("RenameInPlaceToggle", () => {
  it("turns rename only on", () => {
    const onChange = vi.fn();
    render(<RenameInPlaceToggle checked={false} disabled={false} onChange={onChange} />);

    fireEvent.click(screen.getByRole("checkbox", { name: "Rename only" }));

    expect(onChange).toHaveBeenCalledWith(true);
  });

  it("stays put while the destination is locked", () => {
    render(<RenameInPlaceToggle checked={true} disabled={true} onChange={vi.fn()} />);

    const toggle = screen.getByRole("checkbox", { name: "Rename only" });
    expect(toggle).toBeChecked();
    expect(toggle).toBeDisabled();
  });
});
