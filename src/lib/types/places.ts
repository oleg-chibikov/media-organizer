type SuggestionKind = "trip" | "outing";

/** A trip or outing folder an earlier run filled, as a later suggestion continues it. */
export interface ContinuedPlacement {
  title: string;
  year: number;
  start: string;
  end: string;
  /** The placed place folders. A place of the same name widens its day layout by them. */
  places: ContinuedPlace[];
  /** Continues every folder of the suggestion's country in its year, at the suggestion's
   * level. `title` is then the country folder. */
  country_wide: boolean;
  /** Month folders whose files of the country move up into the year folder. */
  lifted: LiftedMonth[];
}

/** A month folder holding placed files of a country that move up to the year folder. */
interface LiftedMonth {
  month_folder: string;
  file_count: number;
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
  /** The country folder outside home's country. A trip is then filed under this country
   * instead of its title, and an outing's place folder sits inside it. */
  country: string | null;
  /** The placed trip or outing this one continues. Its title and year win. */
  continues: ContinuedPlacement | null;
  /** The month folder the place folders go in, as `<year>/<month_folder>`, when the scan is
   * a month folder of the root. Null puts them at year level. */
  month_folder: string | null;
}

/** The place folder a planned file goes to. The frontend passes it back unchanged. */
export interface Placement {
  kind: SuggestionKind;
  year: number;
  title: string;
  place: string | null;
  country?: string | null;
  month_folder?: string | null;
}
