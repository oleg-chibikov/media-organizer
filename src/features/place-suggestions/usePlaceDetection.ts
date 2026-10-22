import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { detectPlaceSuggestions } from "../../lib/api";
import type { FileRecord, HomeLocation, MetadataResult, PlaceSuggestion } from "../../lib/types";
import { metadataResultsFor } from "../metadata";
import { allMetadataRead } from "./allMetadataRead";

// Lets the last metadata batches land before the backend looks at every file.
const DETECTION_DELAY_MS = 1000;

export interface PlaceDetectionOptions {
  // Continued trips are matched under this folder's destination.
  selectedFolder: string | null;
  rows: FileRecord[];
  metadataByFileId: Record<string, MetadataResult>;
  scanCompleted: boolean;
  metadataBusy: boolean;
  enabled: boolean;
  // Trips are measured from home, so a new home asks again.
  home: HomeLocation | null;
  reportError: (message: string) => void;
}

// Asks the backend for place folders once per scan and home, after every file's metadata is read.
export function usePlaceDetection(options: PlaceDetectionOptions) {
  const { selectedFolder, rows, metadataByFileId, scanCompleted, metadataBusy, enabled, home } =
    options;
  const [suggestions, setSuggestions] = useState<PlaceSuggestion[]>([]);
  const [generation, setGeneration] = useState(0);
  const generationRef = useRef(0);
  const detectedKeyRef = useRef<string | null>(null);
  const latestRef = useRef(options);
  useEffect(() => {
    latestRef.current = options;
  });

  const metadataRead = useMemo(
    () => allMetadataRead(rows, metadataByFileId),
    [rows, metadataByFileId]
  );
  const ready =
    enabled &&
    selectedFolder !== null &&
    scanCompleted &&
    !metadataBusy &&
    metadataRead &&
    rows.length > 0;
  const homeKey = home ? `${home.latitude},${home.longitude}` : "none";
  const key = `${generation}:${rows.length}:${homeKey}`;

  useEffect(() => {
    if (!ready || detectedKeyRef.current === key) {
      return;
    }
    const timer = setTimeout(() => {
      detectedKeyRef.current = key;
      const latest = latestRef.current;
      const results = metadataResultsFor(latest.rows, latest.metadataByFileId);
      // A reset or a new home while the backend works makes the answer out of date.
      const isCurrent = () =>
        detectedKeyRef.current === key && generationRef.current === generation;
      if (latest.selectedFolder === null) {
        return;
      }
      detectPlaceSuggestions(latest.selectedFolder, latest.rows, results).then(
        (found) => {
          if (isCurrent()) {
            setSuggestions(found);
          }
        },
        (error: unknown) => {
          if (isCurrent()) {
            latestRef.current.reportError(`Failed to suggest place folders: ${String(error)}`);
          }
        }
      );
    }, DETECTION_DELAY_MS);
    return () => clearTimeout(timer);
  }, [generation, key, ready]);

  const reset = useCallback(() => {
    generationRef.current += 1;
    setGeneration(generationRef.current);
    setSuggestions([]);
  }, []);

  return { suggestions, reset };
}
