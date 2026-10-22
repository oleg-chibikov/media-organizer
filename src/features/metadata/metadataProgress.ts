import type { FileIdMap } from "../../lib/fileIdMap";
import type { FileRecord, MetadataResult } from "../../lib/types";

export interface MetadataProgress {
  read: number;
  total: number;
}

// Only photos and videos get metadata, so other rows don't hold detection back.
export function metadataProgress(
  rows: FileRecord[],
  metadataByFileId: FileIdMap<MetadataResult>
): MetadataProgress {
  let read = 0;
  let total = 0;
  for (const row of rows) {
    if (row.kind === "photo" || row.kind === "video") {
      total += 1;
      if (metadataByFileId.has(row.id)) {
        read += 1;
      }
    }
  }
  return { read, total };
}
