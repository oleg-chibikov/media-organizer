import type { FileRecord, MetadataResult } from "../../lib/types";

export interface MetadataProgress {
  read: number;
  total: number;
}

// Only photos and videos get metadata, so other rows don't hold detection back.
export function metadataProgress(
  rows: FileRecord[],
  metadataByFileId: Record<string, MetadataResult>
): MetadataProgress {
  let read = 0;
  let total = 0;
  for (const row of rows) {
    if (row.kind === "photo" || row.kind === "video") {
      total += 1;
      if (row.id in metadataByFileId) {
        read += 1;
      }
    }
  }
  return { read, total };
}
