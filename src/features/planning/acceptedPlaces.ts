import type { PlaceSuggestion } from "../../lib/types";

// The place folder names of each accepted file. A file absent from the map goes to date folders.
function placeFolderKeys(suggestions: PlaceSuggestion[]): Map<string, string> {
  const keys = new Map<string, string>();
  for (const suggestion of suggestions) {
    const base = suggestion.kind === "trip" ? [suggestion.title] : [];
    for (const place of suggestion.places) {
      const key = JSON.stringify([...base, place.name]);
      place.file_ids.forEach((fileId) => keys.set(fileId, key));
    }
    suggestion.root_file_ids.forEach((fileId) => keys.set(fileId, JSON.stringify(base)));
  }
  return keys;
}

export interface AcceptedPlaces {
  suggestions: PlaceSuggestion[];
  keys: Map<string, string>;
}

export const acceptPlaces = (suggestions: PlaceSuggestion[]): AcceptedPlaces => ({
  suggestions,
  keys: placeFolderKeys(suggestions)
});
