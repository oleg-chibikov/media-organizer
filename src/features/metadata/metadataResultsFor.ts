import type { FileIdMap } from "../../lib/fileIdMap";
import type { FileRecord, MetadataResult } from "../../lib/types";

// The metadata read so far for the rows, in row order.
export function metadataResultsFor(
  rows: FileRecord[],
  metadataByFileId: FileIdMap<MetadataResult>
): MetadataResult[] {
  return rows
    .map((row) => metadataByFileId.get(row.id))
    .filter((result): result is MetadataResult => result !== undefined);
}
