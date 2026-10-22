import { Button } from "../../components/design-system/Button";
import type { PlaceSuggestionsState } from "./usePlaceSuggestions";

interface EditPlacesButtonProps {
  places: Pick<PlaceSuggestionsState, "suggestions" | "openReview">;
  // Confirmed moves keep their folders, as do the settings.
  disabled: boolean;
}

// Opens the edit dialog once detection found trips or outings.
export function EditPlacesButton({ places, disabled }: EditPlacesButtonProps) {
  const count = places.suggestions.length;
  if (count === 0) {
    return null;
  }
  return (
    <Button className="px-2 py-1 text-sm" disabled={disabled} onClick={places.openReview}>
      Edit places ({count})…
    </Button>
  );
}
