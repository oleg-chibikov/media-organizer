import type { DetectionStatus, PlaceSuggestion } from "../../lib/types";
import { noPlacesReason } from "./noPlacesReason";
import { placeBannerLabel } from "./suggestionLabels";

const CREDIT = "Place names from GeoNames and © OpenStreetMap contributors.";

// The tooltip of the "Group by place" box: what it does, what it found, and the data credit.
export function placesHint(
  status: DetectionStatus | null,
  suggestions: PlaceSuggestion[],
  locating: boolean
): string {
  const found = locating
    ? "Finding places..."
    : suggestions.length > 0
      ? placeBannerLabel(suggestions)
      : `${noPlacesReason(status)}.`;
  return `Trips and outings go to place folders. ${found}\n${CREDIT}`;
}
