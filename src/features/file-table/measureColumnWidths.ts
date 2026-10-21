import type { ColumnSizingState } from "@tanstack/react-table";
import { FILE_TABLE_COLUMNS, type FileColumnKey, type FileTableRow } from "./fileTableColumns";

export const COLUMN_HORIZONTAL_PADDING = 28;
const MIN_COLUMN_WIDTH = 50;
const MONO_CELL_FONT = "12px ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace";
const BODY_CELL_FONT =
  "14px system-ui, -apple-system, Segoe UI, Roboto, Helvetica, Arial, sans-serif";
const MONO_COLUMN_KEYS = new Set<FileColumnKey>(["source", "destination"]);

function createMeasureContext(): CanvasRenderingContext2D | null {
  if (typeof document === "undefined") {
    return null;
  }
  return document.createElement("canvas").getContext("2d");
}

function measureText(context: CanvasRenderingContext2D, value: string, font: string): number {
  context.font = font;
  return Math.ceil(context.measureText(value).width);
}

function measureHeaderMinWidths(): Record<FileColumnKey, number> {
  const context = createMeasureContext();
  const widths = {} as Record<FileColumnKey, number>;
  for (const column of FILE_TABLE_COLUMNS) {
    const headerWidth = context ? measureText(context, column.label, BODY_CELL_FONT) : 0;
    widths[column.key] = Math.max(MIN_COLUMN_WIDTH, headerWidth + COLUMN_HORIZONTAL_PADDING);
  }
  return widths;
}

export const HEADER_MIN_WIDTHS = measureHeaderMinWidths();

export const DEFAULT_COLUMN_SIZING: ColumnSizingState = Object.fromEntries(
  FILE_TABLE_COLUMNS.map((column) => [column.key, COLUMN_HORIZONTAL_PADDING])
);

// Fits each column to its widest cell, but keeps the header readable.
export function measureAutoSizedColumns(rows: FileTableRow[]): Record<FileColumnKey, number> {
  const widths = { ...HEADER_MIN_WIDTHS };
  const context = rows.length > 0 ? createMeasureContext() : null;
  if (!context) {
    return widths;
  }
  for (const column of FILE_TABLE_COLUMNS) {
    const font = MONO_COLUMN_KEYS.has(column.key) ? MONO_CELL_FONT : BODY_CELL_FONT;
    const cellMaxWidth = rows.reduce(
      (maxWidth, row) => Math.max(maxWidth, measureText(context, row[column.key].value, font)),
      0
    );
    widths[column.key] = Math.max(
      HEADER_MIN_WIDTHS[column.key],
      cellMaxWidth + COLUMN_HORIZONTAL_PADDING
    );
  }
  return widths;
}
