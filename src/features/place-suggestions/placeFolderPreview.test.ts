import { describe, expect, it } from "vitest";
import { placeFolderPreview } from "./placeFolderPreview";
import { OUTING, TRIP, TRIP_DATES } from "./testSuggestions";

const dateOf = (dates: Record<string, string>) => (fileId: string) => dates[fileId];

describe("placeFolderPreview", () => {
  it("lists trip places with day folders, then the trip root", () => {
    expect(placeFolderPreview(TRIP, dateOf(TRIP_DATES))).toEqual([
      { path: "2025/United States/Yosemite/10-28", count: 1 },
      { path: "2025/United States/Yosemite/10-29", count: 1 },
      { path: "2025/United States/San Francisco", count: 1 },
      { path: "2025/United States/10-30", count: 1 }
    ]);
  });

  it("puts a one day outing straight into its place folder", () => {
    expect(placeFolderPreview(OUTING, dateOf(TRIP_DATES))).toEqual([
      { path: "2025/Royal Botanic Gardens", count: 2 }
    ]);
  });

  it("keeps MM-DD across a month boundary", () => {
    const dates = { ...TRIP_DATES, y1: "2025-10-31 09:00:00", y2: "2025-11-02 09:00:00" };
    const trip = { ...TRIP, places: [{ ...TRIP.places[0], start: dates.y1, end: dates.y2 }] };
    expect(
      placeFolderPreview(trip, dateOf(dates))
        .slice(0, 2)
        .map((row) => row.path)
    ).toEqual(["2025/United States/Yosemite/10-31", "2025/United States/Yosemite/11-02"]);
  });

  it("uses month and day folders past 31 days", () => {
    const dates = { ...TRIP_DATES, y1: "2025-09-03 09:00:00", y2: "2025-10-29 09:00:00" };
    const trip = { ...TRIP, places: [{ ...TRIP.places[0], start: dates.y1, end: dates.y2 }] };
    expect(
      placeFolderPreview(trip, dateOf(dates))
        .slice(0, 2)
        .map((row) => row.path)
    ).toEqual(["2025/United States/Yosemite/9/3", "2025/United States/Yosemite/10/29"]);
  });

  it("widens the span by a root file outside the suggested dates", () => {
    const dates = { ...TRIP_DATES, r1: "2025-10-31 08:00:00" };
    const root = placeFolderPreview(TRIP, dateOf(dates))[3];
    expect(root).toEqual({ path: "2025/United States/10-31", count: 1 });
  });
});
