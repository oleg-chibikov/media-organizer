import { useCallback, useEffect, useRef, type RefObject } from "react";
import { executeMovePlan } from "../../lib/api";
import type { FileRecord, MetadataResult, MovePlan, MoveResult } from "../../lib/types";
import { releaseFailedBatch } from "./moveDispatchCompletion";
import {
  createMoveDispatchQueue,
  enqueueReadyPlans,
  MOVE_DISPATCH_CONCURRENCY,
  takeMoveBatch
} from "./moveDispatchQueue";
import type { MoveExecutionState } from "./moveExecutionState";

interface MoveDispatchOptions {
  selectedFolder: string | null;
  rowByIdRef: RefObject<Map<string, FileRecord>>;
  metadataByFileId: Record<string, MetadataResult>;
  movePlansByFileId: Record<string, MovePlan>;
  moveResultsByFileId: Record<string, MoveResult>;
  executionConfirmed: boolean;
  updateExecution: (patch: Partial<MoveExecutionState>) => void;
  reportError: (message: string) => void;
}

// Owns the dispatch queue and sends ready plans to the backend one batch at a time.
export function useMoveDispatch({
  selectedFolder,
  rowByIdRef,
  metadataByFileId,
  movePlansByFileId,
  moveResultsByFileId,
  executionConfirmed,
  updateExecution,
  reportError
}: MoveDispatchOptions) {
  const queueRef = useRef(createMoveDispatchQueue());
  const confirmedRef = useRef(false);
  const pausedRef = useRef(false);
  const pumpRef = useRef<() => void>(() => undefined);

  const pump = useCallback(() => {
    if (!confirmedRef.current || !selectedFolder || pausedRef.current) {
      return;
    }
    const queue = queueRef.current;
    while (queue.activeRequests < MOVE_DISPATCH_CONCURRENCY && queue.pending.length > 0) {
      const batch = takeMoveBatch(queue, rowByIdRef.current, metadataByFileId, movePlansByFileId);
      if (batch.fileIds.length === 0) {
        continue;
      }
      updateExecution({ executing: true, completed: false });
      void executeMovePlan(selectedFolder, batch.files, batch.metadata, batch.plans).catch(
        (error: unknown) => {
          reportError(`Failed to execute move plan: ${String(error)}`);
          updateExecution(releaseFailedBatch(queueRef.current, batch.fileIds, error));
          queueMicrotask(() => pumpRef.current());
        }
      );
    }
  }, [
    metadataByFileId,
    movePlansByFileId,
    reportError,
    rowByIdRef,
    selectedFolder,
    updateExecution
  ]);

  useEffect(() => {
    pumpRef.current = pump;
  }, [pump]);

  useEffect(() => {
    enqueueReadyPlans(queueRef.current, movePlansByFileId, moveResultsByFileId);
    if (executionConfirmed) {
      pumpRef.current();
    }
  }, [executionConfirmed, movePlansByFileId, moveResultsByFileId]);

  const restartQueue = useCallback((confirmed: boolean) => {
    queueRef.current = createMoveDispatchQueue();
    confirmedRef.current = confirmed;
    pausedRef.current = false;
  }, []);
  const resetDispatch = useCallback(() => restartQueue(false), [restartQueue]);
  const startDispatch = useCallback(() => {
    restartQueue(true);
    enqueueReadyPlans(queueRef.current, movePlansByFileId, {});
  }, [movePlansByFileId, restartQueue]);

  const togglePaused = useCallback((): boolean => {
    pausedRef.current = !pausedRef.current;
    if (!pausedRef.current) {
      queueMicrotask(() => pumpRef.current());
    }
    return pausedRef.current;
  }, []);

  return { queueRef, pumpRef, resetDispatch, startDispatch, togglePaused };
}
