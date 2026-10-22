import type { PlaceSuggestion } from "../../lib/types";

export const TRIP: PlaceSuggestion = {
  id: "trip-1",
  kind: "trip",
  year: 2025,
  title: "United States",
  start: "2025-10-28 09:00:00",
  end: "2025-10-30 18:00:00",
  file_count: 4,
  places: [
    {
      name: "Yosemite",
      place_kind: "park",
      day_layout: "month_day",
      start: "2025-10-28 09:00:00",
      end: "2025-10-29 17:00:00",
      file_ids: ["y1", "y2"]
    },
    {
      name: "San Francisco",
      place_kind: "city",
      day_layout: "single",
      start: "2025-10-29 19:00:00",
      end: "2025-10-29 20:00:00",
      file_ids: ["sf"]
    }
  ],
  root_file_ids: ["r1"],
  continues: null
};

// The trip above, continuing a placed trip whose San Francisco folder holds Oct 27.
export const CONTINUED_TRIP: PlaceSuggestion = {
  ...TRIP,
  continues: {
    title: "United States",
    year: 2025,
    start: "2025-10-20 10:00:00",
    end: "2025-10-27 18:00:00",
    places: [{ name: "San Francisco", start: "2025-10-27 10:00:00", end: "2025-10-27 18:00:00" }]
  }
};

export const OUTING: PlaceSuggestion = {
  id: "outing-1",
  kind: "outing",
  year: 2025,
  title: "Royal Botanic Gardens",
  start: "2025-03-01 10:00:00",
  end: "2025-03-01 12:00:00",
  file_count: 2,
  places: [
    {
      name: "Royal Botanic Gardens",
      place_kind: "garden",
      day_layout: "single",
      start: "2025-03-01 10:00:00",
      end: "2025-03-01 12:00:00",
      file_ids: ["g1", "g2"]
    }
  ],
  root_file_ids: [],
  continues: null
};

export const TRIP_DATES: Record<string, string> = {
  y1: "2025-10-28 09:00:00",
  y2: "2025-10-29 17:00:00",
  sf: "2025-10-29 19:00:00",
  r1: "2025-10-30 18:00:00",
  g1: "2025-03-01 10:00:00",
  g2: "2025-03-01 12:00:00"
};
