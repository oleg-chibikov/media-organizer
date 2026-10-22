import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { GroupByPlaceButton } from "./GroupByPlaceButton";
import { OUTING, TRIP } from "./testSuggestions";

afterEach(cleanup);

const idle = {
  locating: false,
  progress: { read: 3, total: 3 },
  suggestions: [],
  status: null,
  openReview: vi.fn()
};

describe("GroupByPlaceButton", () => {
  it("shows progress while locating", () => {
    render(
      <GroupByPlaceButton places={{ ...idle, locating: true, progress: { read: 2, total: 5 } }} />
    );
    expect(screen.getByRole("status")).toHaveTextContent("Finding locations… 2/5");
  });

  it("opens the review with the number of places", () => {
    const openReview = vi.fn();
    render(<GroupByPlaceButton places={{ ...idle, suggestions: [TRIP, OUTING], openReview }} />);
    fireEvent.click(screen.getByRole("button", { name: "Group by place (2)" }));
    expect(openReview).toHaveBeenCalledOnce();
  });

  it("stays focusable and says why there is nothing to group", () => {
    const openReview = vi.fn();
    const status = { kind: "all_near_home", home: "Sydney" } as const;
    render(<GroupByPlaceButton places={{ ...idle, status, openReview }} />);
    const button = screen.getByRole("button", { name: "Group by place" });

    expect(button).toHaveAttribute("aria-disabled", "true");
    expect(button).toHaveAccessibleDescription("All files are near home (Sydney)");
    fireEvent.click(button);
    expect(openReview).not.toHaveBeenCalled();
  });
});
