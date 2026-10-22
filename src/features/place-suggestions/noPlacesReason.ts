import type { DetectionStatus } from "../../lib/types";

// Why grouping by place finds no folders. `null` means detection didn't answer,
// because the scan stopped early or the lookup failed.
export function noPlacesReason(status: DetectionStatus | null): string {
  switch (status?.kind) {
    case undefined:
      return "Locations weren't checked";
    case "no_gps":
      return "No GPS in these files";
    case "home_unknown":
      return "Set home to find trips";
    case "all_near_home":
      return `All files are near home (${status.home})`;
    case "too_few_geotagged":
      return "Too few geotagged files";
    case "found":
      return "No trips or outings";
  }
}
