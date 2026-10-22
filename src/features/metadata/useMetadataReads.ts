import { useCallback, useState, type RefObject } from "react";
import { setMetadataPaused, type MetadataListeners } from "../../lib/api";
import type { FileRecord, MetadataResult } from "../../lib/types";
import { useMetadataResults } from "./useMetadataResults";

interface MetadataReadsOptions {
  // The scan on screen. Its ref changes as soon as the scan starts, before the next render.
  scanId: number | null;
  scanIdRef: RefObject<number | null>;
  rowByIdRef: RefObject<Map<string, FileRecord>>;
  reportError: (message: string) => void;
  onMetadataRead: (rows: FileRecord[], results: MetadataResult[]) => void;
}

// Holds the dates the backend reads while it scans. Events of a replaced scan are dropped.
export function useMetadataReads({
  scanId,
  scanIdRef,
  rowByIdRef,
  reportError,
  onMetadataRead
}: MetadataReadsOptions) {
  const results = useMetadataResults(onMetadataRead);
  const { addRead, flushReads } = results;
  const [finishedScanId, setFinishedScanId] = useState<number | null>(null);

  const events: MetadataListeners = {
    onResults: ({ scan_id, results: read }) => {
      if (scan_id !== scanIdRef.current) {
        return;
      }
      // The backend sends a file's date after its row, so every row is already known.
      const rows = read
        .map((result) => rowByIdRef.current.get(result.file_id))
        .filter((row): row is FileRecord => row !== undefined);
      addRead({ rows, results: read });
    },
    onFinished: ({ scan_id }) => {
      if (scan_id === scanIdRef.current) {
        // Planning hears of the last dates before reading counts as done.
        flushReads();
        setFinishedScanId(scan_id);
      }
    }
  };

  // Reads already running finish; the next ones wait for the resume.
  // Stable, since the session calls it from an effect.
  const setPaused = useCallback(
    (paused: boolean) => {
      setMetadataPaused(paused).catch((error: unknown) =>
        reportError(`Failed to ${paused ? "pause" : "resume"} reading dates: ${String(error)}`)
      );
    },
    [reportError]
  );

  return {
    metadataByFileId: results.metadataByFileId,
    reading: scanId !== null && finishedScanId !== scanId,
    events,
    addResults: results.addResults,
    removeResults: results.removeResults,
    setPaused,
    resetMetadata: results.clearResults
  };
}
