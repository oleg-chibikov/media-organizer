import { useCallback, useEffect, useRef, useState } from "react";
import type { FileRecord } from "../../lib/types";

export function useScanRows() {
  const [rows, setRows] = useState<FileRecord[]>([]);
  const seenRowIdsRef = useRef<Set<string>>(new Set());
  const rowByIdRef = useRef<Map<string, FileRecord>>(new Map());

  useEffect(() => {
    rowByIdRef.current = new Map(rows.map((row) => [row.id, row]));
  }, [rows]);

  const addRow = useCallback((record: FileRecord): boolean => {
    if (seenRowIdsRef.current.has(record.id)) {
      return false;
    }
    seenRowIdsRef.current.add(record.id);
    setRows((prev) => [...prev, record]);
    return true;
  }, []);

  const resetRows = useCallback(() => {
    seenRowIdsRef.current = new Set();
    rowByIdRef.current = new Map();
    setRows([]);
  }, []);

  return { rows, rowByIdRef, addRow, resetRows };
}
