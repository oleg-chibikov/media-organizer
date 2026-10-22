import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { PlaceSuggestionPrompt } from "./PlaceSuggestionPrompt";
import { OUTING, TRIP } from "./testSuggestions";

afterEach(cleanup);

describe("PlaceSuggestionPrompt", () => {
  it("opens the review from a button", () => {
    const onReview = vi.fn();
    render(<PlaceSuggestionPrompt suggestions={[TRIP]} onReview={onReview} onDismiss={vi.fn()} />);
    fireEvent.click(
      screen.getByRole("button", { name: "Trip to United States: 4 files, 3 days. Review folders" })
    );
    expect(onReview).toHaveBeenCalledOnce();
  });

  it("names an outing", () => {
    render(<PlaceSuggestionPrompt suggestions={[OUTING]} onReview={vi.fn()} onDismiss={vi.fn()} />);
    expect(
      screen.getByRole("button", { name: "Royal Botanic Gardens: 2 files. Review folders" })
    ).toBeInTheDocument();
  });

  it("counts several suggestions", () => {
    const suggestions = [TRIP, OUTING];
    render(
      <PlaceSuggestionPrompt suggestions={suggestions} onReview={vi.fn()} onDismiss={vi.fn()} />
    );
    expect(screen.getByRole("button", { name: "2 place folders suggested. Review" })).toBeVisible();
  });

  it("dismisses from its own button", () => {
    const onDismiss = vi.fn();
    render(<PlaceSuggestionPrompt suggestions={[TRIP]} onReview={vi.fn()} onDismiss={onDismiss} />);
    fireEvent.click(screen.getByRole("button", { name: "Dismiss place folder suggestions" }));
    expect(onDismiss).toHaveBeenCalledOnce();
  });

  it("stays hidden without suggestions", () => {
    const { container } = render(
      <PlaceSuggestionPrompt suggestions={[]} onReview={vi.fn()} onDismiss={vi.fn()} />
    );
    expect(container).toBeEmptyDOMElement();
  });
});
