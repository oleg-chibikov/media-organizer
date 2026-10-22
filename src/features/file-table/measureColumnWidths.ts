import type { ColumnSizingState } from "@tanstack/react-table";
import { FILE_TABLE_COLUMNS, type FileColumnKey, type FileTableRow } from "./fileTableColumns";

const COLUMN_HORIZONTAL_PADDING = 28;
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

// Most cells keep their text between passes, so each width is measured once.
const MAX_CACHED_WIDTHS = 5_000;
// Text this much shorter than the longest cell can't be the widest, even in a proportional font.
const CANDIDATE_LENGTH_RATIO = 0.6;
const MAX_CANDIDATES = 200;
const widthCache = new Map<string, number>();

function measureText(context: CanvasRenderingContext2D, value: string, font: string): number {
  const key = `${font}\n${value}`;
  const cached = widthCache.get(key);
  if (cached !== undefined) {
    return cached;
  }
  if (widthCache.size >= MAX_CACHED_WIDTHS) {
    widthCache.clear();
  }
  context.font = font;
  const width = Math.ceil(context.measureText(value).width);
  widthCache.set(key, width);
  return width;
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

// The cells that may be widest, so a pass over thousands of rows measures a few hundred.
function widthCandidates(rows: FileTableRow[], key: FileColumnKey): string[] {
  const longest = rows.reduce((max, row) => Math.max(max, row[key].value.length), 0);
  const minLength = Math.floor(longest * CANDIDATE_LENGTH_RATIO);
  const candidates = new Set<string>();
  for (const row of rows) {
    if (row[key].value.length >= minLength) {
      candidates.add(row[key].value);
    }
  }
  return [...candidates].sort((a, b) => b.length - a.length).slice(0, MAX_CANDIDATES);
}

// Fits each column to its widest cell, but keeps the header readable.
export function measureAutoSizedColumns(rows: FileTableRow[]): Record<FileColumnKey, number> {
  const widths = { ...HEADER_MIN_WIDTHS };
  const context = rows.length > 0 ? createMeasureContext() : null;
  if (!context) {
    return widths;
  }
  for (const column of FILE_TABLE_COLUMNS) {
    const font = MONO_COLUMN_KEYS.has(column.key) ? MONO_CELL_FONT : BODY_CELL_FONT;
    const cellMaxWidth = widthCandidates(rows, column.key).reduce(
      (maxWidth, value) => Math.max(maxWidth, measureText(context, value, font)),
      0
    );
    widths[column.key] = Math.max(
      HEADER_MIN_WIDTHS[column.key],
      cellMaxWidth + COLUMN_HORIZONTAL_PADDING
    );
  }
  return widths;
}
