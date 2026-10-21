import type { ColumnSizingState } from "@tanstack/react-table";
import { useCallback, useEffect, useRef, useState } from "react";
import type { TableColumn } from "./types";

const RESIZING_BODY_CLASSES = ["cursor-col-resize", "select-none"];
// Rows stream in one event at a time; measuring every cell per event froze large tables.
export const AUTO_SIZE_INTERVAL_MS = 250;

interface ColumnSizingOptions<TColumnKey extends string> {
  rowCount: number;
  columns: readonly TableColumn<TColumnKey>[];
  defaultColumnSizing: ColumnSizingState;
  onAutoSizeColumns: () => ColumnSizingState;
}

// Auto-sizes columns as rows arrive, except columns the person resized by hand.
export function useColumnSizing<TColumnKey extends string>({
  rowCount,
  columns,
  defaultColumnSizing,
  onAutoSizeColumns
}: ColumnSizingOptions<TColumnKey>) {
  const manuallyResizedColumnsRef = useRef<Set<TColumnKey>>(new Set());
  const [columnSizing, setColumnSizing] = useState<ColumnSizingState>(defaultColumnSizing);

  const markColumnAsResized = useCallback((columnKey: TColumnKey) => {
    manuallyResizedColumnsRef.current.add(columnKey);
    document.body.classList.add(...RESIZING_BODY_CLASSES);
  }, []);

  const setColumnWidth = useCallback((columnKey: TColumnKey, width: number) => {
    manuallyResizedColumnsRef.current.add(columnKey);
    setColumnSizing((previous) => ({ ...previous, [columnKey]: width }));
  }, []);

  const latestRef = useRef({ columns, onAutoSizeColumns });
  const autoSizeTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  useEffect(() => {
    latestRef.current = { columns, onAutoSizeColumns };
  });

  const autoSize = useCallback(() => {
    autoSizeTimerRef.current = null;
    const { columns: latestColumns, onAutoSizeColumns: measure } = latestRef.current;
    const desiredWidths = measure();
    setColumnSizing((previous) => {
      const next: ColumnSizingState = { ...previous };
      const resizedByHand = manuallyResizedColumnsRef.current;
      const changed = latestColumns.filter(
        ({ key }) => !resizedByHand.has(key) && next[key] !== desiredWidths[key]
      );
      changed.forEach(({ key }) => (next[key] = desiredWidths[key]));
      return changed.length > 0 ? next : previous;
    });
  }, []);

  useEffect(() => {
    if (rowCount > 0 && autoSizeTimerRef.current === null) {
      autoSizeTimerRef.current = setTimeout(autoSize, AUTO_SIZE_INTERVAL_MS);
    }
  }, [autoSize, onAutoSizeColumns, rowCount]);

  useEffect(
    () => () => {
      if (autoSizeTimerRef.current !== null) {
        clearTimeout(autoSizeTimerRef.current);
        autoSizeTimerRef.current = null;
      }
    },
    []
  );

  useEffect(() => {
    const stopResizing = () => {
      document.body.classList.remove(...RESIZING_BODY_CLASSES);
    };

    window.addEventListener("pointerup", stopResizing);
    return () => {
      window.removeEventListener("pointerup", stopResizing);
      stopResizing();
    };
  }, []);

  return { columnSizing, setColumnSizing, markColumnAsResized, setColumnWidth };
}
