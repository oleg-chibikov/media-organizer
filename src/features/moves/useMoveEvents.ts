import type { RefObject } from "react";
import { attachMoveListeners, useBackendEvents } from "../../lib/api";
import type { FileRecord, MoveResult } from "../../lib/types";
import { countMoveResult, type MoveCounts } from "./moveCounts";
import { isDispatchDrained, recordMoveResult, type MoveDispatchQueue } from "./moveDispatchQueue";
import { patchMoveExecution, type MoveExecutionState } from "./moveExecutionState";
import { formatMoveProgress } from "./formatMoveProgress";

interface MoveEventsOptions {
  queueRef: RefObject<MoveDispatchQueue>;
  pumpRef: RefObject<() => void>;
  rowByIdRef: RefObject<Map<string, FileRecord>>;
  setExecution: (update: (previous: MoveExecutionState) => MoveExecutionState) => void;
  setMoveResultsByFileId: (
    update: (previous: Record<string, MoveResult>) => Record<string, MoveResult>
  ) => void;
  setMoveCounts: (update: (previous: MoveCounts) => MoveCounts) => void;
  reportError: (message: string) => void;
}

export function useMoveEvents({
  queueRef,
  pumpRef,
  rowByIdRef,
  setExecution,
  setMoveResultsByFileId,
  setMoveCounts,
  reportError
}: MoveEventsOptions) {
  const updateExecution = (patch: Partial<MoveExecutionState>) =>
    setExecution((previous) => patchMoveExecution(previous, patch));

  const applyMoveResult = (result: MoveResult) => {
    const isFirstResult = recordMoveResult(queueRef.current, result.file_id);
    setMoveResultsByFileId((previous) => ({ ...previous, [result.file_id]: result }));
    setExecution((previous) =>
      previous.activeFileId === result.file_id
        ? patchMoveExecution(previous, { activeFileId: null })
        : previous
    );
    if (!isFirstResult) {
      return;
    }
    setMoveCounts((previous) => countMoveResult(previous, result.result_status));
    if (result.result_status === "failed" && result.error) {
      reportError(`Move error: ${result.error}`);
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
      onFile: ({ result }) => applyMoveResult(result),
      onFinished: ({ results }) => {
        results.forEach(applyMoveResult);
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
