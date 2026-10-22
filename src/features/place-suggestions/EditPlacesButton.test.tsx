import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { EditPlacesButton } from "./EditPlacesButton";
import { OUTING, TRIP } from "./testSuggestions";

afterEach(cleanup);

describe("EditPlacesButton", () => {
  it("opens the edit dialog with the count of places", () => {
    const openReview = vi.fn();
    render(
      <EditPlacesButton places={{ suggestions: [TRIP, OUTING], openReview }} disabled={false} />
    );

    fireEvent.click(screen.getByRole("button", { name: "Edit places (2)…" }));

    expect(openReview).toHaveBeenCalledOnce();
  });

  it("stays hidden until detection finds a place", () => {
    render(<EditPlacesButton places={{ suggestions: [], openReview: vi.fn() }} disabled={false} />);

    expect(screen.queryByRole("button")).toBeNull();
  });
});
