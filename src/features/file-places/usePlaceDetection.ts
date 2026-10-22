import type { FileIdMap } from "../../lib/fileIdMap";
import { useCallback, useEffect, useMemo, useRef, useState, type RefObject } from "react";
import { detectFilePlaces } from "../../lib/api";
import type { FileRecord, HomeLocation, MetadataResult, PlaceDetection } from "../../lib/types";
import { metadataProgress } from "../metadata";
import { usePlacePreview } from "./usePlacePreview";

// Lets the last metadata batches land before the backend looks at every file.
const DETECTION_DELAY_MS = 1000;

export interface PlaceDetectionOptions {
  selectedFolder: string | null;
  // The backend reads the files and dates of this scan by id.
  scanIdRef: RefObject<number | null>;
  rows: FileRecord[];
  metadataByFileId: FileIdMap<MetadataResult>;
  scanCompleted: boolean;
  // A stopped scan is never detected, so it shows no progress either.
  scanCancelled: boolean;
  metadataBusy: boolean;
  // Places outside home's country get a country folder, so a new home asks again.
  home: HomeLocation | null;
  reportError: (message: string) => void;
}

// Asks the backend for place folders once per scan and home, after every file's metadata is read.
// Until then previews show where files will likely go.
export function usePlaceDetection(options: PlaceDetectionOptions) {
  const { selectedFolder, rows, metadataByFileId, scanCompleted, metadataBusy, home } = options;
  const [detection, setDetection] = useState<PlaceDetection | null>(null);
  const [preview, setPreview] = useState(false);
  // The key the last answer, or failure, belongs to.
  const [answeredKey, setAnsweredKey] = useState<string | null>(null);
  const [generation, setGeneration] = useState(0);
  const generationRef = useRef(0);
  const detectedKeyRef = useRef<string | null>(null);
  const latestRef = useRef(options);
  useEffect(() => {
    latestRef.current = options;
  });

  const progress = useMemo(
    () => metadataProgress(rows, metadataByFileId),
    [rows, metadataByFileId]
  );
  const hasMedia = selectedFolder !== null && progress.total > 0;
  const ready = hasMedia && scanCompleted && !metadataBusy && progress.read === progress.total;
  const homeKey = home ? `${home.latitude},${home.longitude}` : "none";
  const key = `${generation}:${rows.length}:${homeKey}`;

  usePlacePreview({
    active: hasMedia && !ready && !options.scanCancelled,
    resetKey: `${generation}:${homeKey}`,
    latestRef,
    onPreview: (found) => {
      setDetection(found);
      setPreview(true);
    }
  });

  useEffect(() => {
    if (!ready || detectedKeyRef.current === key) {
      return;
    }
    const timer = setTimeout(() => {
      detectedKeyRef.current = key;
      const latest = latestRef.current;
      // A reset or a new home while the backend works makes the answer out of date.
      const isCurrent = () =>
        detectedKeyRef.current === key && generationRef.current === generation;
      const scanId = latest.scanIdRef.current;
      if (latest.selectedFolder === null || scanId === null) {
        return;
      }
      const fileIds = latest.rows.map((row) => row.id);
      detectFilePlaces(scanId, fileIds, false).then(
        (found) => {
          if (isCurrent()) {
            setDetection(found);
            setPreview(false);
            setAnsweredKey(key);
          }
        },
        (error: unknown) => {
          if (isCurrent()) {
            // Files go to date folders, as with no places, rather than to a preview's guess.
            setDetection(null);
            setPreview(false);
            setAnsweredKey(key);
            latestRef.current.reportError(`Failed to find places: ${String(error)}`);
          }
        }
      );
    }, DETECTION_DELAY_MS);
    return () => clearTimeout(timer);
  }, [generation, key, ready]);

  const reset = useCallback(() => {
    generationRef.current += 1;
    setGeneration(generationRef.current);
    setDetection(null);
    setPreview(false);
  }, []);

  const locating = hasMedia && !options.scanCancelled && answeredKey !== key;
  return {
    detection,
    // The detection came from a preview, so it can still change.
    preview,
    locating,
    // Detection answered, failed, or has no files to look at.
    settled: !locating && (scanCompleted || options.scanCancelled),
    progress,
    reset
  };
}
