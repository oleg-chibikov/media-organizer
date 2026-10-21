import type { RefObject } from "react";
import { attachMoveListeners, useBackendEvents } from "../../lib/api";
import type { FileRecord, MoveResult } from "../../lib/types";
import { FINISHED_RUN, isDispatchDrained, recordMoveResult } from "./moveDispatchCompletion";
import type { MoveDispatchQueue } from "./moveDispatchQueue";
import { formatMoveProgress } from "./formatMoveProgress";
import type { MoveStatus } from "./useMoveStatus";

interface MoveEventsOptions {
  queueRef: RefObject<MoveDispatchQueue>;
  pumpRef: RefObject<() => void>;
  rowByIdRef: RefObject<Map<string, FileRecord>>;
  status: Pick<MoveStatus, "recordResults" | "updateExecution">;
  reportError: (message: string) => void;
}

// Applies backend move progress and results to the dispatch queue and the visible status.
export function useMoveEvents({
  queueRef,
  pumpRef,
  rowByIdRef,
  status,
  reportError
}: MoveEventsOptions) {
  const { recordResults, updateExecution } = status;
  // A failed file shows its error in its row and in the failed count.
  const applyMoveResults = (results: MoveResult[]) => {
    recordResults(results);
    for (const result of results) {
      recordMoveResult(queueRef.current, result.file_id);
    }
  };

  useBackendEvents(
    attachMoveListeners,
    {
      onProgress: (payload) => {
        const rowLabel = payload.file_id
          ? rowByIdRef.current.get(payload.file_id)?.relative_path
          : undefined;
        updateExecution({
          statusMessage: formatMoveProgress(payload, rowLabel),
          activeFileId: payload.file_id
        });
      },
      onFile: ({ result }) => applyMoveResults([result]),
      onFinished: ({ results }) => {
        applyMoveResults(results);
        const queue = queueRef.current;
        queue.activeRequests = Math.max(0, queue.activeRequests - 1);
        if (isDispatchDrained(queue)) {
          updateExecution(FINISHED_RUN);
        }
        queueMicrotask(() => pumpRef.current());
      }
    },
    (error) => reportError(`Failed to attach move listeners: ${String(error)}`)
  );
}
