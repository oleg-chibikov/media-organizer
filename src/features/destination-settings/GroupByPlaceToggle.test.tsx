import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { GroupByPlaceToggle } from "./GroupByPlaceToggle";

afterEach(cleanup);

const renderToggle = (renamesInPlace: boolean, onChange = vi.fn()) =>
  render(
    <GroupByPlaceToggle
      checked={true}
      renamesInPlace={renamesInPlace}
      disabled={false}
      placesHint="3 files have a place."
      onChange={onChange}
    />
  );

describe("GroupByPlaceToggle", () => {
  it("turns grouping off and tells what detection found", () => {
    const onChange = vi.fn();
    renderToggle(false, onChange);

    const toggle = screen.getByRole("checkbox", { name: "Group by place" });
    fireEvent.click(toggle);

    expect(onChange).toHaveBeenCalledWith(false);
    expect(toggle.closest("label")).toHaveAttribute("title", "3 files have a place.");
  });

  it("shows off and locked while rename only is on", () => {
    renderToggle(true);

    const toggle = screen.getByRole("checkbox", { name: "Group by place" });
    expect(toggle).not.toBeChecked();
    expect(toggle).toBeDisabled();
  });
});
