import { describe, expect, it } from "vitest";
import { placeBannerLabel } from "./suggestionLabels";
import { CONTINUED_TRIP, OUTING, TRIP } from "./testSuggestions";

describe("placeBannerLabel", () => {
  it("names a trip with its days and an outing by its place", () => {
    expect(placeBannerLabel([TRIP, OUTING])).toBe(
      "Found 2 places: Trip to United States (3 days), Royal Botanic Gardens."
    );
  });

  it("names the placed trip a suggestion continues", () => {
    expect(placeBannerLabel([CONTINUED_TRIP])).toBe(
      "Found 1 place: Continues United States (Oct 20, 2025–Oct 27, 2025)."
    );
  });

  it("counts the places past the third", () => {
    const outings = [1, 2, 3, 4].map((n) => ({ ...OUTING, id: `o${n}`, title: `Park ${n}` }));
    expect(placeBannerLabel(outings)).toBe("Found 4 places: Park 1, Park 2, Park 3 and 1 more.");
  });
});
