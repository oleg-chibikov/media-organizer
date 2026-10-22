import type { FileRecord, MetadataReadRequest, MetadataResult } from "../../lib/types";

const METADATA_BATCH_SIZE = 8;

export interface MetadataQueue {
  pending: string[];
  queuedIds: Set<string>;
  requestedIds: Set<string>;
  activeRequests: number;
}

export function createMetadataQueue(): MetadataQueue {
  return { pending: [], queuedIds: new Set(), requestedIds: new Set(), activeRequests: 0 };
}

function hasReadableMetadata(row: FileRecord): boolean {
  return row.kind === "photo" || row.kind === "video";
}

// Mutates the queue: appends media rows that have no metadata yet, up to the capacity.
export function enqueueRowsNeedingMetadata(
  queue: MetadataQueue,
  rows: FileRecord[],
  metadataByFileId: Record<string, MetadataResult>,
  capacity: number
): void {
  for (const row of rows) {
    if (!hasReadableMetadata(row) || metadataByFileId[row.id]) {
      continue;
    }
    if (queue.requestedIds.has(row.id) || queue.queuedIds.has(row.id)) {
      continue;
    }
    if (queue.pending.length >= capacity) {
      break;
    }
    queue.pending.push(row.id);
    queue.queuedIds.add(row.id);
  }
}

export function toMetadataRequest(row: FileRecord): MetadataReadRequest {
  return { file_id: row.id, source_path: row.source_path, file_kind: row.kind };
}

// Mutates the queue: takes up to one batch of known rows and marks them as requested.
export function takeMetadataBatch(
  queue: MetadataQueue,
  rowById: Map<string, FileRecord>
): FileRecord[] {
  const batch: FileRecord[] = [];
  while (batch.length < METADATA_BATCH_SIZE && queue.pending.length > 0) {
    // One splice per batch instead of one shift per file.
    const fileIds = queue.pending.splice(0, METADATA_BATCH_SIZE - batch.length);
    for (const fileId of fileIds) {
      queue.queuedIds.delete(fileId);
      const row = rowById.get(fileId);
      if (row) {
        queue.requestedIds.add(fileId);
        batch.push(row);
      }
    }
  }
  return batch;
}
