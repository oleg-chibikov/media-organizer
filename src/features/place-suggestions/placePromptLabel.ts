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

export function placePromptLabel(suggestions: PlaceSuggestion[]): string {
  const [only] = suggestions;
  if (suggestions.length === 1 && only) {
    const heading = only.continues ? continuedLabel(only.continues) : suggestionHeading(only);
    return `${heading}: ${suggestionSize(only)}. Review folders`;
  }
  return `${plural(suggestions.length, "place folder")} suggested. Review`;
}
