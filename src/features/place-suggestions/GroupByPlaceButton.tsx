import { useId } from "react";
import { Button } from "../../components/design-system/Button";
import { Spinner } from "../../components/design-system/Spinner";
import { noPlacesReason } from "./noPlacesReason";
import type { PlaceSuggestionsState } from "./usePlaceSuggestions";

type PlaceButtonSource = Pick<
  PlaceSuggestionsState,
  "locating" | "progress" | "suggestions" | "status" | "openReview"
>;

// The toolbar entry to place folders: progress while locating, then the review or why not.
export function GroupByPlaceButton({ places }: { places: PlaceButtonSource }) {
  const reasonId = useId();
  if (places.locating) {
    const { read, total } = places.progress;
    return (
      <span role="status" aria-live="polite" className="flex items-center gap-2 text-sm">
        <Spinner />
        Finding locations… {read}/{total}
      </span>
    );
  }
  const count = places.suggestions.length;
  if (count > 0) {
    return (
      <Button className="px-3 py-1.5 text-sm" onClick={places.openReview}>
        Group by place ({count})
      </Button>
    );
  }
  const reason = noPlacesReason(places.status);
  // Stays focusable, so keyboard and screen reader users reach the reason too.
  return (
    <span className="flex items-center gap-2">
      <Button
        aria-disabled="true"
        aria-describedby={reasonId}
        title={reason}
        className="cursor-not-allowed px-3 py-1.5 text-sm text-slate-400 hover:bg-slate-800"
      >
        Group by place
      </Button>
      <span id={reasonId} className="text-xs text-slate-400">
        {reason}
      </span>
    </span>
  );
}
