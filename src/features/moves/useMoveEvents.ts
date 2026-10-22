import type { RefObject } from "react";
import { attachMoveListeners, useBackendEvents } from "../../lib/api";
import type { FileRecord, MoveProgressEvent, MoveResult } from "../../lib/types";
import { useBufferedFlush } from "../../lib/useBufferedFlush";
import {
  finishBatch,
  FINISHED_RUN,
  isDispatchDrained,
  recordMoveResult
} from "./moveDispatchCompletion";
import type { MoveDispatchQueue } from "./moveDispatchQueue";
import { formatMoveProgress } from "./formatMoveProgress";
import type { MoveStatus } from "./useMoveStatus";

// A run sends 3 events per file; the table redraws at most this often.
const MOVE_FLUSH_MS = 200;

type MoveEvent = { result: MoveResult } | { progress: MoveProgressEvent };

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
      onFile: ({ result }) => {
        recordMoveResult(queueRef.current, result.file_id);
        buffer.add({ result });
      },
      onFinished: ({ results }) => {
        buffer.flush();
        const queue = queueRef.current;
        results.forEach((result) => recordMoveResult(queue, result.file_id));
        recordResults(results);
        finishBatch(queue);
        if (isDispatchDrained(queue)) {
          updateExecution(FINISHED_RUN);
        }
        queueMicrotask(() => pumpRef.current());
      }
    },
    (error) => reportError(`Failed to attach move listeners: ${String(error)}`)
  );
}
