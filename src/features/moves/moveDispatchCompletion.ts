import type { MoveDispatchQueue } from "./moveDispatchQueue";
import type { MoveExecutionState } from "./moveExecutionState";

export const FINISHED_RUN: Partial<MoveExecutionState> = {
  executing: false,
  activeFileId: null,
  statusMessage: null,
  completed: true
};

// Mutates the queue: frees a batch the backend refused so it can be queued again.
// Returns the execution patch that reports the failure.
export function releaseFailedBatch(
  queue: MoveDispatchQueue,
  fileIds: string[],
  error: unknown
): Partial<MoveExecutionState> {
  queue.activeRequests = Math.max(0, queue.activeRequests - 1);
  for (const fileId of fileIds) {
    queue.inflightIds.delete(fileId);
    queue.trackedIds.delete(fileId);
  }
  return {
    statusMessage: `Move dispatch failed: ${String(error)}`,
    activeFileId: null,
    ...(isDispatchDrained(queue) ? { executing: false, completed: true } : {})
  };
}

// Mutates the queue: returns false when the file already had a result.
export function recordMoveResult(queue: MoveDispatchQueue, fileId: string): boolean {
  const isFirstResult = !queue.completedIds.has(fileId);
  queue.inflightIds.delete(fileId);
  queue.completedIds.add(fileId);
  queue.trackedIds.add(fileId);
  return isFirstResult;
}

export function isDispatchDrained(queue: MoveDispatchQueue): boolean {
  return queue.activeRequests === 0 && queue.pending.length === 0 && !queue.awaitingPlans;
}
