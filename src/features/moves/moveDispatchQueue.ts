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

export interface MoveBatch {
  fileIds: string[];
  files: FileRecord[];
  metadata: MetadataResult[];
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
  plansByFileId: Record<string, MovePlan>,
  resultsByFileId: Record<string, MoveResult>
): void {
  for (const plan of Object.values(plansByFileId)) {
    const fileId = plan.file_id;
    if (plan.plan_status !== "ready" || queue.trackedIds.has(fileId)) {
      continue;
    }
    queue.trackedIds.add(fileId);
    if (resultsByFileId[fileId] !== undefined) {
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
  metadataByFileId: Record<string, MetadataResult>,
  plansByFileId: Record<string, MovePlan>
): MoveBatch {
  const batch: MoveBatch = { fileIds: [], files: [], metadata: [], plans: [] };
  while (queue.pending.length > 0) {
    const fileId = queue.pending.shift();
    if (!fileId) {
      continue;
    }
    queue.queuedIds.delete(fileId);
    if (queue.inflightIds.has(fileId) || queue.completedIds.has(fileId)) {
      continue;
    }
    const row = rowById.get(fileId);
    const metadata = metadataByFileId[fileId];
    const plan = plansByFileId[fileId];
    if (!row || !metadata || !plan || plan.plan_status !== "ready") {
      queue.trackedIds.delete(fileId);
      continue;
    }
    batch.fileIds.push(fileId);
    batch.files.push(row);
    batch.metadata.push(metadata);
    batch.plans.push(plan);
  }
  if (batch.fileIds.length > 0) {
    queue.activeRequests += 1;
    batch.fileIds.forEach((fileId) => queue.inflightIds.add(fileId));
  }
  return batch;
}
