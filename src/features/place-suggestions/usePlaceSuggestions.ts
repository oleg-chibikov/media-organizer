import { useCallback, useMemo, useState } from "react";
import type { PlaceSuggestion } from "../../lib/types";
import { acceptedSuggestions, type PlaceChoice } from "./placeChoice";
import { usePlaceDetection, type PlaceDetectionOptions } from "./usePlaceDetection";

interface PlaceSuggestionsOptions extends PlaceDetectionOptions {
  // Files already moving keep the folders they were confirmed with.
  locked: boolean;
  onApply: (accepted: PlaceSuggestion[]) => void;
}

const NO_SUGGESTIONS: PlaceSuggestion[] = [];
const NO_FILE_PLACES: Record<string, string> = {};

export function usePlaceSuggestions({
  locked,
  onApply,
  ...detectionOptions
}: PlaceSuggestionsOptions) {
  const {
    detection,
    locating,
    progress,
    reset: resetDetection
  } = usePlaceDetection(detectionOptions);
  // Kept across scans: a suggestion id stays the same for the same files.
  const [dismissedIds, setDismissedIds] = useState<ReadonlySet<string>>(() => new Set());
  const [choices, setChoices] = useState<Record<string, PlaceChoice>>({});
  const [reviewOpen, setReviewOpen] = useState(false);

  const found = detection?.suggestions ?? NO_SUGGESTIONS;
  const suggestions = locked ? NO_SUGGESTIONS : found;
  // Dismissing hides the banner only. The toolbar button still opens the review.
  const bannerVisible = useMemo(
    () => suggestions.some((suggestion) => !dismissedIds.has(suggestion.id)),
    [dismissedIds, suggestions]
  );

  const reset = useCallback(() => {
    resetDetection();
    setChoices({});
    setReviewOpen(false);
  }, [resetDetection]);

  return {
    suggestions,
    bannerVisible,
    locating,
    progress,
    status: detection?.status ?? null,
    filePlaces: detection?.file_places ?? NO_FILE_PLACES,
    choices,
    reviewOpen: reviewOpen && suggestions.length > 0,
    openReview: () => setReviewOpen(true),
    closeReview: () => setReviewOpen(false),
    dismiss: () =>
      setDismissedIds((previous) => new Set([...previous, ...suggestions.map((s) => s.id)])),
    apply: (edited: Record<string, PlaceChoice>) => {
      const next = { ...choices, ...edited };
      setChoices(next);
      setReviewOpen(false);
      onApply(acceptedSuggestions(found, next));
    },
    reset
  };
}

export type PlaceSuggestionsState = ReturnType<typeof usePlaceSuggestions>;
