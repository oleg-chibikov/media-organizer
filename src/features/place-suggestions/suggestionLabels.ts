import type { PlaceSuggestion } from "../../lib/types";
import { continuedLabel } from "./continuedLabel";

const DAY_MS = 86_400_000;
// The backend titles a trip "Trip" when it finds no country.
const UNKNOWN_TRIP_TITLE = "Trip";

const plural = (count: number, word: string) => `${count} ${word}${count === 1 ? "" : "s"}`;

function dayCount(suggestion: PlaceSuggestion): number {
  const first = Date.parse(suggestion.start.slice(0, 10));
  const last = Date.parse(suggestion.end.slice(0, 10));
  return Math.round((last - first) / DAY_MS) + 1;
}

export function suggestionHeading(suggestion: PlaceSuggestion): string {
  if (suggestion.kind === "outing") {
    return suggestion.title;
  }
  return suggestion.title === UNKNOWN_TRIP_TITLE ? "Trip" : `Trip to ${suggestion.title}`;
}

export function suggestionSize(suggestion: PlaceSuggestion): string {
  const files = plural(suggestion.file_count, "file");
  return suggestion.kind === "trip" ? `${files}, ${plural(dayCount(suggestion), "day")}` : files;
}

// How many names the banner lists before it counts the rest.
const BANNER_NAMES = 3;

function bannerItem(suggestion: PlaceSuggestion): string {
  if (suggestion.continues) {
    return continuedLabel(suggestion.continues);
  }
  const heading = suggestionHeading(suggestion);
  return suggestion.kind === "trip"
    ? `${heading} (${plural(dayCount(suggestion), "day")})`
    : heading;
}

// One line, for example "Found 2 places: Trip to Italy (5 days), Royal Botanic Gardens."
export function placeBannerLabel(suggestions: PlaceSuggestion[]): string {
  const names = suggestions.slice(0, BANNER_NAMES).map(bannerItem);
  const rest = suggestions.length - names.length;
  const list = rest > 0 ? `${names.join(", ")} and ${rest} more` : names.join(", ");
  return `Found ${plural(suggestions.length, "place")}: ${list}.`;
}
