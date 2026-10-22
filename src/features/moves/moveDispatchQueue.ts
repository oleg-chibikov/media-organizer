import type { FileIdMap } from "../../lib/fileIdMap";
import type { FileRecord, MetadataResult, MovePlan, MoveResult } from "../../lib/types";

export const MOVE_DISPATCH_CONCURRENCY = 1;

export interface MoveDispatchQueue {
  pending: string[];
  queuedIds: Set<string>;
  inflightIds: Set<string>;
  completedIds: Set<string>;
  trackedIds: Set<string>;
  activeRequests: number;
  // Files still being checked can join a confirmed run, so it isn't finished yet.
  awaitingPlans: boolean;
}

// The backend looks each plan's file and date up in the scan, so a batch carries plans only.
export interface MoveBatch {
  fileIds: string[];
  plans: MovePlan[];
}

export function createMoveDispatchQueue(): MoveDispatchQueue {
  return {
    pending: [],
    queuedIds: new Set(),
    inflightIds: new Set(),
    completedIds: new Set(),
    trackedIds: new Set(),
    activeRequests: 0,
    awaitingPlans: false
  };
}

// Mutates the queue: tracks every ready plan once and queues those without a result.
export function enqueueReadyPlans(
  queue: MoveDispatchQueue,
  plansByFileId: FileIdMap<MovePlan>,
  resultsByFileId: FileIdMap<MoveResult>
): void {
  for (const plan of plansByFileId.values()) {
    const fileId = plan.file_id;
    if (plan.plan_status !== "ready" || queue.trackedIds.has(fileId)) {
      continue;
    }
    queue.trackedIds.add(fileId);
    if (resultsByFileId.get(fileId) !== undefined) {
      queue.completedIds.add(fileId);
      continue;
    }
    queue.queuedIds.add(fileId);
    queue.pending.push(fileId);
  }
}

// Mutates the queue: drains every dispatchable file into one batch and marks it in flight.
export function takeMoveBatch(
  queue: MoveDispatchQueue,
  rowById: Map<string, FileRecord>,
  metadataByFileId: FileIdMap<MetadataResult>,
  plansByFileId: FileIdMap<MovePlan>
): MoveBatch {
  const batch: MoveBatch = { fileIds: [], plans: [] };
  const pending = queue.pending;
  queue.pending = [];
  for (const fileId of pending) {
    queue.queuedIds.delete(fileId);
    if (queue.inflightIds.has(fileId) || queue.completedIds.has(fileId)) {
      continue;
    }
    const plan = plansByFileId.get(fileId);
    if (
      !rowById.has(fileId) ||
      !metadataByFileId.has(fileId) ||
      !plan ||
      plan.plan_status !== "ready"
    ) {
      queue.trackedIds.delete(fileId);
      continue;
    }
    batch.fileIds.push(fileId);
    batch.plans.push(plan);
  }
  if (batch.fileIds.length > 0) {
    queue.activeRequests += 1;
    batch.fileIds.forEach((fileId) => queue.inflightIds.add(fileId));
  }
  return batch;
}
