import type { MoveDispatchQueue } from "./moveDispatchQueue";

// Mutates the queue: frees a batch the backend refused so it can be queued again.
export function releaseFailedBatch(queue: MoveDispatchQueue, fileIds: string[]): void {
  queue.activeRequests = Math.max(0, queue.activeRequests - 1);
  for (const fileId of fileIds) {
    queue.inflightIds.delete(fileId);
    queue.trackedIds.delete(fileId);
  }
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
  return queue.activeRequests === 0 && queue.pending.length === 0;
}
