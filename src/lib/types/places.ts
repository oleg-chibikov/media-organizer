type SuggestionKind = "trip" | "outing";

/** A trip or outing folder an earlier run filled, as a later suggestion continues it. */
export interface ContinuedPlacement {
  title: string;
  year: number;
  start: string;
  end: string;
  /** The placed place folders. A place of the same name widens its day layout by them. */
  places: ContinuedPlace[];
}

interface ContinuedPlace {
  name: string;
  start: string;
  end: string;
}

/** How a place folder splits its files: none, `MM-DD` or `<month>/<day>`. */
type DayLayout = "single" | "month_day" | "month_folder_day";

interface PlaceGroup {
  name: string;
  place_kind: string;
  day_layout: DayLayout;
  start: string;
  end: string;
  file_ids: string[];
}

/** A suggested trip or outing folder. Dates use the `chosen_date` format. */
export interface PlaceSuggestion {
  id: string;
  kind: SuggestionKind;
  year: number;
  title: string;
  start: string;
  end: string;
  file_count: number;
  places: PlaceGroup[];
  /** Trip files that belong to no place folder. */
  root_file_ids: string[];
  /** The placed trip or outing this one continues. Its title and year win. */
  continues: ContinuedPlacement | null;
}

/** The place folder a planned file goes to. The frontend passes it back unchanged. */
export interface Placement {
  kind: SuggestionKind;
  year: number;
  title: string;
  place: string | null;
}
