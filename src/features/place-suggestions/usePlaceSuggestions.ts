import { useEffect, useMemo, useRef, useState } from "react";
import type { PlaceSuggestion } from "../../lib/types";
import { acceptedSuggestions, type PlaceChoice } from "./placeChoice";
import { placesHint } from "./placesHint";
import { usePlaceDetection, type PlaceDetectionOptions } from "./usePlaceDetection";

interface PlaceSuggestionsOptions extends PlaceDetectionOptions {
  // Off sends every file to date folders without waiting for detection.
  groupByPlace: boolean;
  // Gets the suggestions the plans should use, each time they change.
  onApply: (accepted: PlaceSuggestion[]) => void;
}

const NO_SUGGESTIONS: PlaceSuggestion[] = [];
const NO_FILE_PLACES: Record<string, string> = {};

// Detects trips and outings, and hands the ones the person kept to planning while grouping is on.
export function usePlaceSuggestions({
  groupByPlace,
  onApply,
  ...detectionOptions
}: PlaceSuggestionsOptions) {
  const {
    detection,
    locating,
    settled,
    progress,
    reset: resetDetection
  } = usePlaceDetection(detectionOptions);
  const suggestions = detection?.suggestions ?? NO_SUGGESTIONS;
  // Kept across scans and toggles: a suggestion id stays the same for the same files.
  const [choices, setChoices] = useState<Record<string, PlaceChoice>>({});
  const [reviewOpen, setReviewOpen] = useState(false);
  const accepted = useMemo(
    () => (groupByPlace ? acceptedSuggestions(suggestions, choices) : NO_SUGGESTIONS),
    [choices, groupByPlace, suggestions]
  );
  const onApplyRef = useRef(onApply);
  // What planning got last. Placed files join the rows and trigger detection again,
  // so an answer with the same suggestions changes no plan.
  const appliedRef = useRef<string | null>(null);
  useEffect(() => {
    onApplyRef.current = onApply;
  });

  useEffect(() => {
    if (groupByPlace && !settled) {
      return;
    }
    const signature = JSON.stringify(accepted);
    if (appliedRef.current !== signature) {
      appliedRef.current = signature;
      onApplyRef.current(accepted);
    }
  }, [accepted, groupByPlace, settled]);

  const status = detection?.status ?? null;
  return {
    locating,
    progress,
    hint: placesHint(status, suggestions, locating),
    filePlaces: detection?.file_places ?? NO_FILE_PLACES,
    suggestions,
    choices,
    // Editing only changes the planned paths. Files move from the Move Files button.
    reviewOpen: reviewOpen && suggestions.length > 0,
    openReview: () => setReviewOpen(true),
    closeReview: () => setReviewOpen(false),
    applyChoices: (edited: Record<string, PlaceChoice>) => {
      setChoices((previous) => ({ ...previous, ...edited }));
      setReviewOpen(false);
    },
    reset: () => {
      setReviewOpen(false);
      appliedRef.current = null;
      resetDetection();
    }
  };
}

export type PlaceSuggestionsState = ReturnType<typeof usePlaceSuggestions>;
