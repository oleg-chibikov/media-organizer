import type { RefObject } from "react";
import { attachMoveListeners, useBackendEvents } from "../../lib/api";
import type { FileRecord, MoveResult } from "../../lib/types";
import { isDispatchDrained, recordMoveResult } from "./moveDispatchCompletion";
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
  const applyMoveResults = (results: MoveResult[]) => {
    recordResults(results);
    for (const result of results) {
      const isFirstResult = recordMoveResult(queueRef.current, result.file_id);
      if (isFirstResult && result.result_status === "failed" && result.error) {
        reportError(`Move error: ${result.error}`);
      }
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
          updateExecution({
            executing: false,
            activeFileId: null,
            statusMessage: null,
            completed: true
          });
        }
        queueMicrotask(() => pumpRef.current());
      }
    },
    (error) => reportError(`Failed to attach move listeners: ${String(error)}`)
  );
}
