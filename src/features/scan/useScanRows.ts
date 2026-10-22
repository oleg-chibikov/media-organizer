import { useCallback, useEffect, useRef, useState } from "react";
import type { FileRecord } from "../../lib/types";

// Each flush re-sorts every row, so a large folder needs few of them. Still feels live.
export const SCAN_ROW_FLUSH_MS = 250;

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

  const resetRows = useCallback(() => {
    cancelScheduledFlush();
    pendingRowsRef.current = [];
    rowByIdRef.current = new Map();
    setRows([]);
  }, [cancelScheduledFlush]);

  useEffect(() => cancelScheduledFlush, [cancelScheduledFlush]);

  return { rows, rowByIdRef, addRow, flushRows, resetRows };
}
