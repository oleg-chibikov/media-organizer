import { useCallback, useMemo, useState } from "react";
import type { PlaceSuggestion } from "../../lib/types";
import { acceptedSuggestions, type PlaceChoice } from "./placeChoice";
import { usePlaceDetection, type PlaceDetectionOptions } from "./usePlaceDetection";

interface PlaceSuggestionsOptions extends PlaceDetectionOptions {
  // Files already moving keep the folders they were confirmed with.
  locked: boolean;
  onApply: (accepted: PlaceSuggestion[]) => void;
}

export function usePlaceSuggestions({
  locked,
  onApply,
  ...detectionOptions
}: PlaceSuggestionsOptions) {
  const { suggestions, reset: resetDetection } = usePlaceDetection(detectionOptions);
  // Kept across scans: a suggestion id stays the same for the same files.
  const [dismissedIds, setDismissedIds] = useState<ReadonlySet<string>>(() => new Set());
  const [choices, setChoices] = useState<Record<string, PlaceChoice>>({});
  const [reviewOpen, setReviewOpen] = useState(false);

  const pending = useMemo(
    () => (locked ? [] : suggestions.filter((suggestion) => !dismissedIds.has(suggestion.id))),
    [dismissedIds, locked, suggestions]
  );

  const reset = useCallback(() => {
    resetDetection();
    setChoices({});
    setReviewOpen(false);
  }, [resetDetection]);

  return {
    pending,
    choices,
    reviewOpen: reviewOpen && pending.length > 0,
    openReview: () => setReviewOpen(true),
    closeReview: () => setReviewOpen(false),
    dismiss: () =>
      setDismissedIds((previous) => new Set([...previous, ...pending.map((s) => s.id)])),
    apply: (edited: Record<string, PlaceChoice>) => {
      const next = { ...choices, ...edited };
      setChoices(next);
      setReviewOpen(false);
      onApply(acceptedSuggestions(suggestions, next));
    },
    reset
  };
}
