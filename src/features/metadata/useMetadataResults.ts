import { useCallback, useState } from "react";
import { FileIdMap } from "../../lib/fileIdMap";
import type { MetadataResult } from "../../lib/types";
import { useBufferedFlush } from "../../lib/useBufferedFlush";

// Batches land every few milliseconds; the table takes them at most this often.
const METADATA_FLUSH_MS = 250;

const fileIdOf = (result: MetadataResult) => result.file_id;

// Holds the date read for each file. A file's own read error shows in its row.
export function useMetadataResults() {
  const [metadataByFileId, setMetadataByFileId] = useState(FileIdMap.empty<MetadataResult>);

  const {
    add: addRead,
    flush: flushReads,
    drop: dropReads
  } = useBufferedFlush<MetadataResult[]>(
    (batches) => setMetadataByFileId((prev) => prev.with(batches.flat(), fileIdOf)),
    METADATA_FLUSH_MS
  );

  // Placed files come with the date their record kept, so they need no read.
  const addResults = useCallback((results: MetadataResult[]) => {
    if (results.length > 0) {
      setMetadataByFileId((prev) => prev.with(results, fileIdOf));
    }
  }, []);

  const removeResults = useCallback((fileIds: string[]) => {
    if (fileIds.length > 0) {
      setMetadataByFileId((prev) => prev.without(fileIds));
    }
  }, []);

  const clearResults = useCallback(() => {
    dropReads();
    setMetadataByFileId(FileIdMap.empty());
  }, [dropReads]);

  return { metadataByFileId, addRead, flushReads, addResults, removeResults, clearResults };
}
