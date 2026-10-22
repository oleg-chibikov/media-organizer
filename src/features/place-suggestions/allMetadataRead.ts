import type { FileRecord, MetadataResult } from "../../lib/types";

// Only photos and videos get metadata, so other rows don't hold detection back.
export function allMetadataRead(
  rows: FileRecord[],
  metadataByFileId: Record<string, MetadataResult>
): boolean {
  return rows.every(
    (row) => (row.kind !== "photo" && row.kind !== "video") || row.id in metadataByFileId
  );
}
