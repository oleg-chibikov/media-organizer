import type { FileRecord, MetadataResult } from "../../lib/types";

// The metadata read so far for the rows, in row order.
export function metadataResultsFor(
  rows: FileRecord[],
  metadataByFileId: Record<string, MetadataResult>
): MetadataResult[] {
  return rows
    .map((row) => metadataByFileId[row.id])
    .filter((result): result is MetadataResult => result !== undefined);
}
