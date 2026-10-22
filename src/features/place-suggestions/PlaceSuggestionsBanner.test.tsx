import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { PlaceSuggestionsBanner } from "./PlaceSuggestionsBanner";
import { OUTING, TRIP } from "./testSuggestions";

afterEach(cleanup);

const found = {
  locating: false,
  bannerVisible: true,
  suggestions: [TRIP, OUTING],
  openReview: vi.fn(),
  dismiss: vi.fn()
};

describe("PlaceSuggestionsBanner", () => {
  it("shows a loader while locations are looked up", () => {
    render(<PlaceSuggestionsBanner places={{ ...found, locating: true }} />);
    expect(screen.getByText("Finding locations…")).toBeInTheDocument();
  });

  it("lists the places and opens the review", () => {
    const openReview = vi.fn();
    render(<PlaceSuggestionsBanner places={{ ...found, openReview }} />);
    expect(
      screen.getByText("Found 2 places: Trip to United States (3 days), Royal Botanic Gardens.")
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Review folders" }));
    expect(openReview).toHaveBeenCalledOnce();
  });

  it("dismisses from its own button", () => {
    const dismiss = vi.fn();
    render(<PlaceSuggestionsBanner places={{ ...found, dismiss }} />);
    fireEvent.click(screen.getByRole("button", { name: "Dismiss place folder suggestions" }));
    expect(dismiss).toHaveBeenCalledOnce();
  });

  it("stays hidden once dismissed", () => {
    const { container } = render(
      <PlaceSuggestionsBanner places={{ ...found, bannerVisible: false }} />
    );
    expect(container).toBeEmptyDOMElement();
  });
});
