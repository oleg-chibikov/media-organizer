type SuggestionKind = "trip" | "outing";

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
}
