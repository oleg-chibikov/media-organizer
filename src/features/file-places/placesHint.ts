import type { DetectionStatus } from "../../lib/types";

const CREDIT = "Place names from GeoNames and © OpenStreetMap contributors.";

// The tooltip of the "Group by place" box: what it does, what it found, and the data credit.
export function placesHint(
  status: DetectionStatus | null,
  placedFiles: number,
  locating: boolean
): string {
  const found = locating
    ? "Finding places..."
    : status?.kind === "found"
      ? `${placedFiles} ${placedFiles === 1 ? "file has" : "files have"} a place.`
      : status?.kind === "no_gps"
        ? "No GPS in these files."
        : "Locations weren't checked.";
  return `Photos and videos go to a folder of their place, inside their country abroad. ${found}\n${CREDIT}`;
}
