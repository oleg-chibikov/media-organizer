import type { RefObject } from "react";
import { attachMoveListeners, useBackendEvents } from "../../lib/api";
import type { FileRecord, MoveProgressEvent, MoveResult } from "../../lib/types";
import { useBufferedFlush } from "../../lib/useBufferedFlush";
import { formatMoveProgress } from "./formatMoveProgress";
import { FINISHED_RUN } from "./moveExecutionState";
import type { MoveStatus } from "./useMoveStatus";

// A run sends 3 events per file; the table redraws at most this often.
const MOVE_FLUSH_MS = 200;

type MoveEvent = { result: MoveResult } | { progress: MoveProgressEvent };

interface MoveEventsOptions {
  scanIdRef: RefObject<number | null>;
  rowByIdRef: RefObject<Map<string, FileRecord>>;
  status: Pick<MoveStatus, "recordResults" | "updateExecution">;
  reportError: (message: string) => void;
}

// Applies the backend's move progress and results to the visible status.
export function useMoveEvents({ scanIdRef, rowByIdRef, status, reportError }: MoveEventsOptions) {
  const { recordResults, updateExecution } = status;

  // Progress goes first, so a file that finished in the same flush isn't shown as moving.
  const buffer = useBufferedFlush<MoveEvent>((events) => {
    const results: MoveResult[] = [];
    let progress: MoveProgressEvent | undefined;
    for (const event of events) {
      if ("result" in event) {
        results.push(event.result);
      } else {
        progress = event.progress;
      }
    }
    if (progress) {
      const rowLabel = progress.file_id
        ? rowByIdRef.current.get(progress.file_id)?.relative_path
        : undefined;
      updateExecution({
        statusMessage: formatMoveProgress(progress, rowLabel),
        activeFileId: progress.file_id
      });
    }
    // A failed file shows its error in its row and in the failed count.
    recordResults(results);
  }, MOVE_FLUSH_MS);

  useBackendEvents(
    attachMoveListeners,
    {
      onProgress: (payload) => buffer.add({ progress: payload }),
      onFile: ({ result }) => buffer.add({ result }),
      onFinished: ({ results }) => {
        buffer.flush();
        recordResults(results);
        updateExecution({ activeFileId: null });
      },
      onDone: ({ scan_id, error }) => {
        if (scan_id !== scanIdRef.current) {
          return;
        }
        buffer.flush();
        updateExecution(FINISHED_RUN);
        if (error) {
          reportError(error);
        }
      }
    },
    (error) => reportError(`Failed to attach move listeners: ${String(error)}`)
  );
}
