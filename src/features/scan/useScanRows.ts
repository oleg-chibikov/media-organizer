import { useCallback, useEffect, useRef, useState } from "react";
import type { FileRecord } from "../../lib/types";

// Long enough to batch a burst of scan events, short enough to feel live.
export const SCAN_ROW_FLUSH_MS = 50;

// Collects streamed scan records and appends them to `rows` in batches, one render per batch.
export function useScanRows() {
  const [rows, setRows] = useState<FileRecord[]>([]);
  const rowByIdRef = useRef<Map<string, FileRecord>>(new Map());
  const pendingRowsRef = useRef<FileRecord[]>([]);
  const flushTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const cancelScheduledFlush = useCallback(() => {
    if (flushTimerRef.current !== null) {
      clearTimeout(flushTimerRef.current);
      flushTimerRef.current = null;
    }
  }, []);

  const flushRows = useCallback(() => {
    cancelScheduledFlush();
    const pendingRows = pendingRowsRef.current;
    if (pendingRows.length === 0) {
      return;
    }
    pendingRowsRef.current = [];
    setRows((previous) => previous.concat(pendingRows));
  }, [cancelScheduledFlush]);

  const addRow = useCallback(
    (record: FileRecord): boolean => {
      if (rowByIdRef.current.has(record.id)) {
        return false;
      }
      rowByIdRef.current.set(record.id, record);
      pendingRowsRef.current.push(record);
      flushTimerRef.current ??= setTimeout(flushRows, SCAN_ROW_FLUSH_MS);
      return true;
    },
    [flushRows]
  );

  // Placed files that leave the plan leave the rows too.
  const removeRows = useCallback((fileIds: string[]) => {
    const gone = new Set(fileIds.filter((fileId) => rowByIdRef.current.delete(fileId)));
    if (gone.size === 0) {
      return;
    }
    pendingRowsRef.current = pendingRowsRef.current.filter((row) => !gone.has(row.id));
    setRows((previous) => previous.filter((row) => !gone.has(row.id)));
  }, []);

  const resetRows = useCallback(() => {
    cancelScheduledFlush();
    pendingRowsRef.current = [];
    rowByIdRef.current = new Map();
    setRows([]);
  }, [cancelScheduledFlush]);

  useEffect(() => cancelScheduledFlush, [cancelScheduledFlush]);

  return { rows, rowByIdRef, addRow, removeRows, flushRows, resetRows };
}
