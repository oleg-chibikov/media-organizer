import { useCallback, useState } from "react";
import type { FileRecord, MetadataResult } from "../../lib/types";
import { useBufferedFlush } from "../../lib/useBufferedFlush";

// Batches land every few milliseconds; the table and planning take them at most this often.
const METADATA_FLUSH_MS = 250;

interface ReadBatch {
  rows: FileRecord[];
  results: MetadataResult[];
}

const byFileId = (results: MetadataResult[]) =>
  Object.fromEntries(results.map((result) => [result.file_id, result]));

// Holds the date read for each file and hands new reads to planning in buffered groups.
export function useMetadataResults(
  onMetadataRead: (rows: FileRecord[], results: MetadataResult[]) => void
) {
  const [metadataByFileId, setMetadataByFileId] = useState<Record<string, MetadataResult>>({});

  // A file's own read error shows in its row, not as an app error.
  const {
    add: addRead,
    flush: flushReads,
    drop: dropReads
  } = useBufferedFlush<ReadBatch>((batches) => {
    const results = batches.flatMap((batch) => batch.results);
    setMetadataByFileId((prev) => ({ ...prev, ...byFileId(results) }));
    onMetadataRead(
      batches.flatMap((batch) => batch.rows),
      results
    );
  }, METADATA_FLUSH_MS);

  // Placed files come with the date their record kept, so they need no read.
  const addResults = useCallback((results: MetadataResult[]) => {
    if (results.length > 0) {
      setMetadataByFileId((prev) => ({ ...prev, ...byFileId(results) }));
    }
  }, []);

  const removeResults = useCallback((fileIds: string[]) => {
    if (fileIds.length > 0) {
      setMetadataByFileId((prev) =>
        Object.fromEntries(Object.entries(prev).filter(([fileId]) => !fileIds.includes(fileId)))
      );
    }
  }, []);

  const clearResults = useCallback(() => {
    dropReads();
    setMetadataByFileId({});
  }, [dropReads]);

  return { metadataByFileId, addRead, flushReads, addResults, removeResults, clearResults };
}
