import { useEffect, useRef } from "react";
import type { FilePlace } from "../../lib/types";
import { placesHint } from "./placesHint";
import { usePlaceDetection, type PlaceDetectionOptions } from "./usePlaceDetection";

interface FilePlacesOptions extends PlaceDetectionOptions {
  // Off sends every file to date folders without waiting for detection.
  groupByPlace: boolean;
  // Gets the places the plans should use, each time they change.
  onApply: (filePlaces: Record<string, FilePlace>) => void;
}

const NO_FILE_PLACES: Record<string, FilePlace> = {};

// Finds the place of each file, and hands the places to planning while grouping is on.
export function useFilePlaces({ groupByPlace, onApply, ...detectionOptions }: FilePlacesOptions) {
  const {
    detection,
    locating,
    settled,
    progress,
    reset: resetDetection
  } = usePlaceDetection(detectionOptions);
  const filePlaces = detection?.file_places ?? NO_FILE_PLACES;
  const applied = groupByPlace ? filePlaces : NO_FILE_PLACES;
  const onApplyRef = useRef(onApply);
  // What planning got last, so a rerender with the same answer changes no plan.
  const appliedRef = useRef<Record<string, FilePlace> | null>(null);
  useEffect(() => {
    onApplyRef.current = onApply;
  });

  useEffect(() => {
    if (groupByPlace && !settled) {
      return;
    }
    if (appliedRef.current !== applied) {
      appliedRef.current = applied;
      onApplyRef.current(applied);
    }
  }, [applied, groupByPlace, settled]);

  return {
    locating,
    progress,
    hint: placesHint(detection?.status ?? null, Object.keys(filePlaces).length, locating),
    filePlaces,
    reset: () => {
      appliedRef.current = null;
      resetDetection();
    }
  };
}
