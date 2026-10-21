import type { ColumnSizingState } from "@tanstack/react-table";
import { useCallback, useEffect, useRef, useState } from "react";
import type { TableColumn } from "./types";

const RESIZING_BODY_CLASSES = ["cursor-col-resize", "select-none"];

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

  useEffect(() => {
    if (rowCount === 0) {
      return;
    }

    const desiredWidths = onAutoSizeColumns();
    setColumnSizing((previous) => {
      let changed = false;
      const next: ColumnSizingState = { ...previous };
      for (const column of columns) {
        const desiredWidth = desiredWidths[column.key];
        if (
          !manuallyResizedColumnsRef.current.has(column.key) &&
          next[column.key] !== desiredWidth
        ) {
          next[column.key] = desiredWidth;
          changed = true;
        }
      }
      return changed ? next : previous;
    });
  }, [columns, onAutoSizeColumns, rowCount]);

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

  return { columnSizing, setColumnSizing, markColumnAsResized };
}
