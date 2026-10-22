import { useEffect, useRef } from "react";
import type { PlaceSuggestion } from "../../lib/types";
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

// Detects trips and outings, and hands them to planning while grouping by place is on.
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
    const accepted = groupByPlace ? suggestions : NO_SUGGESTIONS;
    const signature = JSON.stringify(accepted);
    if (appliedRef.current !== signature) {
      appliedRef.current = signature;
      onApplyRef.current(accepted);
    }
  }, [groupByPlace, settled, suggestions]);

  const status = detection?.status ?? null;
  return {
    locating,
    progress,
    hint: placesHint(status, suggestions, locating),
    filePlaces: detection?.file_places ?? NO_FILE_PLACES,
    reset: () => {
      appliedRef.current = null;
      resetDetection();
    }
  };
}
