import { useEffect, useRef, type RefObject } from "react";
import type { FileIdMap } from "../../lib/fileIdMap";
import { detectFilePlaces } from "../../lib/api";
import type { FileRecord, MetadataResult, PlaceDetection } from "../../lib/types";

const PREVIEW_INTERVAL_MS = 2000;

export interface PreviewSource {
  scanIdRef: RefObject<number | null>;
  rows: FileRecord[];
  metadataByFileId: FileIdMap<MetadataResult>;
}

interface PlacePreviewOptions {
  // Previews run only while the scan and its dates are still read.
  active: boolean;
  // A new scan or home drops the previews of the old one.
  resetKey: string;
  latestRef: RefObject<PreviewSource>;
  onPreview: (detection: PlaceDetection) => void;
}

// Asks for places every 2 s while dates are read, so places and destinations show early.
export function usePlacePreview({ active, resetKey, latestRef, onPreview }: PlacePreviewOptions) {
  const onPreviewRef = useRef(onPreview);
  useEffect(() => {
    onPreviewRef.current = onPreview;
  });

  useEffect(() => {
    if (!active) {
      return;
    }
    let stale = false;
    let busy = false;
    let lastAsked: FileIdMap<MetadataResult> | null = null;
    const timer = setInterval(() => {
      const { scanIdRef, rows, metadataByFileId } = latestRef.current;
      const scanId = scanIdRef.current;
      if (busy || scanId === null || metadataByFileId.size === 0) {
        return;
      }
      if (metadataByFileId === lastAsked) {
        return;
      }
      busy = true;
      lastAsked = metadataByFileId;
      const fileIds = rows.map((row) => row.id);
      detectFilePlaces(scanId, fileIds, true)
        .then(
          (found) => {
            if (!stale) {
              onPreviewRef.current(found);
            }
          },
          // A failed preview leaves the places to the final detection.
          () => undefined
        )
        .finally(() => {
          busy = false;
        });
    }, PREVIEW_INTERVAL_MS);
    return () => {
      stale = true;
      clearInterval(timer);
    };
  }, [active, resetKey, latestRef]);
}
