import { useCallback, useState } from "react";
import type { FileRecord, MetadataResult } from "../../lib/types";
import { FileIdMap } from "../../lib/fileIdMap";
import { useBufferedFlush } from "../../lib/useBufferedFlush";

// Batches land every few milliseconds; the table and planning take them at most this often.
const METADATA_FLUSH_MS = 250;

interface ReadBatch {
  rows: FileRecord[];
  results: MetadataResult[];
}

const fileIdOf = (result: MetadataResult) => result.file_id;

// Holds the date read for each file and hands new reads to planning in buffered groups.
export function useMetadataResults(
  onMetadataRead: (rows: FileRecord[], results: MetadataResult[]) => void
) {
  const [metadataByFileId, setMetadataByFileId] = useState(FileIdMap.empty<MetadataResult>);

  // A file's own read error shows in its row, not as an app error.
  const {
    add: addRead,
    flush: flushReads,
    drop: dropReads
  } = useBufferedFlush<ReadBatch>((batches) => {
    const results = batches.flatMap((batch) => batch.results);
    setMetadataByFileId((prev) => prev.with(results, fileIdOf));
    onMetadataRead(
      batches.flatMap((batch) => batch.rows),
      results
    );
  }, METADATA_FLUSH_MS);

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
