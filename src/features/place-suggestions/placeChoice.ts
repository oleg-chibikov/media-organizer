import type { PlaceSuggestion } from "../../lib/types";
import { folderNameError } from "./folderNameError";

// What the person picked for one suggestion in the edit dialog.
export interface PlaceChoice {
  use: boolean;
  title: string;
  placeNames: string[];
}

export function freshChoice(suggestion: PlaceSuggestion): PlaceChoice {
  return {
    use: true,
    title: suggestion.title,
    placeNames: suggestion.places.map((place) => place.name)
  };
}

// The folder a continued suggestion shares with the placed one: the trip, or the outing's place.
const sharedFolder = (suggestion: PlaceSuggestion) =>
  suggestion.kind === "trip" ? suggestion.title : suggestion.places[0]?.name;

// A trip abroad is filed under its country, so only a trip at home has a title to edit.
export const hasEditableTitle = (suggestion: PlaceSuggestion) =>
  suggestion.kind === "trip" && suggestion.country === null;

// The suggestion with the edited names, as the backend plans it. A renamed trip or outing
// no longer continues the placed folder, so it starts a new one. A country wide continuation
// stays, since the country folder keeps its name.
export function withChoiceNames(suggestion: PlaceSuggestion, choice: PlaceChoice): PlaceSuggestion {
  const edited = {
    ...suggestion,
    title: hasEditableTitle(suggestion) ? choice.title.trim() : suggestion.title,
    places: suggestion.places.map((place, index) => ({
      ...place,
      name: (choice.placeNames[index] ?? place.name).trim()
    }))
  };
  const renamed = sharedFolder(edited) !== sharedFolder(suggestion);
  return renamed && !suggestion.continues?.country_wide ? { ...edited, continues: null } : edited;
}

export function isChoiceValid(suggestion: PlaceSuggestion, choice: PlaceChoice): boolean {
  if (!choice.use) {
    return true;
  }
  const names = hasEditableTitle(suggestion)
    ? [choice.title, ...choice.placeNames]
    : choice.placeNames;
  return names.every((name) => folderNameError(name) === null);
}

// A suggestion the person didn't edit goes to its place folders as found.
export function acceptedSuggestions(
  suggestions: PlaceSuggestion[],
  choices: Record<string, PlaceChoice>
): PlaceSuggestion[] {
  return suggestions.flatMap((suggestion) => {
    const choice = choices[suggestion.id] ?? freshChoice(suggestion);
    return choice.use ? [withChoiceNames(suggestion, choice)] : [];
  });
}
