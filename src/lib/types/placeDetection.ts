import type { PlaceSuggestion } from "./places";

/** Why detection found suggestions, or the first reason it found none. */
export type DetectionStatus =
  | { kind: "found" }
  | { kind: "no_gps" }
  | { kind: "home_unknown" }
  /** Every geotagged file is near `home`, the home city. */
  | { kind: "all_near_home"; home: string }
  | { kind: "too_few_geotagged" };

/** What `detect_place_suggestions` found for one scan. */
export interface PlaceDetection {
  suggestions: PlaceSuggestion[];
  /** A short place label per geotagged photo or video, by file id. */
  file_places: Record<string, string>;
  status: DetectionStatus;
}
