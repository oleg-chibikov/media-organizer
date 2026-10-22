import { describe, expect, it } from "vitest";
import { continuedLabel, liftedLabels } from "./continuedLabel";
import { CONTINUED_TRIP, LIFTED_TRIP } from "./testSuggestions";

describe("liftedLabels", () => {
  it("says which month folder moves up to the year", () => {
    expect(liftedLabels(LIFTED_TRIP.continues!)).toEqual([
      "Moves 2026/5/USA into 2026/USA (34 files)"
    ]);
  });

  it("names each month and counts one file", () => {
    const placed = {
      ...LIFTED_TRIP.continues!,
      lifted: [
        { month_folder: "3", file_count: 1 },
        { month_folder: "5", file_count: 34 }
      ]
    };
    expect(liftedLabels(placed)[0]).toBe("Moves 2026/3/USA into 2026/USA (1 file)");
  });

  it("says nothing for a continuation that lifts no month", () => {
    expect(liftedLabels(CONTINUED_TRIP.continues!)).toEqual([]);
  });

  it("names the country a country wide continuation continues", () => {
    expect(continuedLabel(LIFTED_TRIP.continues!)).toBe("Continues USA (May 3, 2026–May 5, 2026)");
  });
});
