import { useVirtualizer } from "@tanstack/react-virtual";
import {
  flexRender,
  getCoreRowModel,
  type ColumnDef,
  type ColumnSizingState,
  useReactTable
} from "@tanstack/react-table";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { FileRecord, MetadataResult, MovePlan, MoveResult } from "../lib/types";

interface FileTableProps {
  rows: FileRecord[];
  metadataByFileId: Record<string, MetadataResult>;
  movePlansByFileId: Record<string, MovePlan>;
  moveResultsByFileId: Record<string, MoveResult>;
  selectedFolder: string | null;
}

const STATUS_LABELS: Record<FileRecord["scan_status"], string> = {
  discovered: "Discovered",
  skipped_processed: "Skipped Processed",
  skipped_unsupported: "Skipped Unsupported",
  scan_error: "Scan Error"
};

const DATE_SOURCE_LABELS: Record<string, string> = {
  SubSecDateTimeOriginal: "Original photo capture time",
  DateTimeOriginal: "Photo capture time",
  MediaCreateDate: "Video creation time",
  TrackCreateDate: "Video track creation time",
  CreateDate: "Embedded media creation time",
  ModifyDate: "Embedded media modified time",
  FileModifyDate: "File modified time (fallback)"
};

const COLUMNS = [
  { key: "source", label: "Source File" },
  { key: "destination", label: "Destination" },
  { key: "status", label: "Status" },
  { key: "kind", label: "Kind" },
  { key: "originalName", label: "Original Name" },
  { key: "metadataDate", label: "Metadata Date" },
  { key: "notes", label: "Notes" }
] as const;

type ColumnKey = (typeof COLUMNS)[number]["key"];

interface CellData {
  value: string;
  title: string;
  className: string;
}

type TableRowData = {
  id: string;
} & Record<ColumnKey, CellData>;

const COLUMN_HORIZONTAL_PADDING = 28;
const MIN_COLUMN_WIDTH = 50;
const MONO_CELL_FONT = "12px ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace";
const BODY_CELL_FONT =
  "14px system-ui, -apple-system, Segoe UI, Roboto, Helvetica, Arial, sans-serif";
const MONO_COLUMN_KEYS = new Set<ColumnKey>(["source", "destination"]);

export function FileTable({
  rows,
  metadataByFileId,
  movePlansByFileId,
  moveResultsByFileId,
  selectedFolder
}: FileTableProps) {
  const parentRef = useRef<HTMLDivElement | null>(null);
  const manuallyResizedColumnsRef = useRef<Set<ColumnKey>>(new Set());

  const [columnSizing, setColumnSizing] = useState<ColumnSizingState>(() =>
    COLUMNS.reduce<ColumnSizingState>((accumulator, column) => {
      accumulator[column.key] = COLUMN_HORIZONTAL_PADDING;
      return accumulator;
    }, {})
  );

  const tableRows = useMemo<TableRowData[]>(
    () =>
      rows.map((row) => {
        const metadata = metadataByFileId[row.id];
        const movePlan = movePlansByFileId[row.id];
        const moveResult = moveResultsByFileId[row.id];

        return {
          id: row.id,
          source: resolveColumnData("source", row, metadata, movePlan, moveResult, selectedFolder),
          destination: resolveColumnData(
            "destination",
            row,
            metadata,
            movePlan,
            moveResult,
            selectedFolder
          ),
          status: resolveColumnData("status", row, metadata, movePlan, moveResult, selectedFolder),
          kind: resolveColumnData("kind", row, metadata, movePlan, moveResult, selectedFolder),
          originalName: resolveColumnData(
            "originalName",
            row,
            metadata,
            movePlan,
            moveResult,
            selectedFolder
          ),
          metadataDate: resolveColumnData(
            "metadataDate",
            row,
            metadata,
            movePlan,
            moveResult,
            selectedFolder
          ),
          notes: resolveColumnData("notes", row, metadata, movePlan, moveResult, selectedFolder)
        };
      }),
    [rows, metadataByFileId, movePlansByFileId, moveResultsByFileId, selectedFolder]
  );

  const tableColumns = useMemo<ColumnDef<TableRowData>[]>(
    () =>
      COLUMNS.map((column) => ({
        id: column.key,
        header: column.label,
        accessorKey: column.key,
        minSize: MIN_COLUMN_WIDTH,
        cell: (context) => {
          const data = context.getValue() as CellData;
          return (
            <div className={data.className} title={data.title}>
              {data.value}
            </div>
          );
        }
      })),
    []
  );

  const table = useReactTable({
    data: tableRows,
    columns: tableColumns,
    getCoreRowModel: getCoreRowModel(),
    columnResizeMode: "onChange",
    enableColumnResizing: true,
    state: { columnSizing },
    onColumnSizingChange: setColumnSizing
  });

  const gridTemplateColumns = table
    .getVisibleLeafColumns()
    .map((column) => `${column.getSize()}px`)
    .join(" ");

  const tableMinWidth = table.getTotalSize();
  const tableWidth = `max(100%, ${tableMinWidth}px)`;

  const rowVirtualizer = useVirtualizer({
    count: tableRows.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 38,
    overscan: 20
  });

  const markColumnAsResized = useCallback((columnKey: ColumnKey) => {
    manuallyResizedColumnsRef.current.add(columnKey);
    document.body.classList.add("cursor-col-resize", "select-none");
  }, []);

  useEffect(() => {
    if (rows.length === 0) {
      return;
    }

    const canvas = document.createElement("canvas");
    const context = canvas.getContext("2d");
    if (!context) {
      return;
    }

    const measureWidth = (value: string, font: string) => {
      context.font = font;
      return Math.ceil(context.measureText(value).width);
    };

    const desiredWidths: Record<ColumnKey, number> = {
      source: COLUMN_HORIZONTAL_PADDING,
      destination: COLUMN_HORIZONTAL_PADDING,
      status: COLUMN_HORIZONTAL_PADDING,
      kind: COLUMN_HORIZONTAL_PADDING,
      originalName: COLUMN_HORIZONTAL_PADDING,
      metadataDate: COLUMN_HORIZONTAL_PADDING,
      notes: COLUMN_HORIZONTAL_PADDING
    };

    COLUMNS.forEach((column) => {
      const headerWidth = measureWidth(column.label, BODY_CELL_FONT);
      const cellMaxWidth = tableRows.reduce((maxWidth, row) => {
        const font = MONO_COLUMN_KEYS.has(column.key) ? MONO_CELL_FONT : BODY_CELL_FONT;
        return Math.max(maxWidth, measureWidth(row[column.key].value, font));
      }, 0);

      desiredWidths[column.key] = Math.max(
        MIN_COLUMN_WIDTH,
        headerWidth + COLUMN_HORIZONTAL_PADDING,
        cellMaxWidth + COLUMN_HORIZONTAL_PADDING
      );
    });

    setColumnSizing((previous) => {
      let changed = false;
      const next: ColumnSizingState = { ...previous };

      for (const column of COLUMNS) {
        if (manuallyResizedColumnsRef.current.has(column.key)) {
          continue;
        }

        const desiredWidth = desiredWidths[column.key];
        if (next[column.key] !== desiredWidth) {
          next[column.key] = desiredWidth;
          changed = true;
        }
      }

      return changed ? next : previous;
    });
  }, [rows.length, tableRows]);

  useEffect(() => {
    const stopResizing = () => {
      document.body.classList.remove("cursor-col-resize", "select-none");
    };

    window.addEventListener("pointerup", stopResizing);

    return () => {
      window.removeEventListener("pointerup", stopResizing);
      document.body.classList.remove("cursor-col-resize", "select-none");
    };
  }, []);

  return (
    <div className="flex h-full flex-col overflow-hidden">
      <div className="min-h-0 flex-1 overflow-x-auto">
        <div className="flex h-full w-full flex-col" style={{ width: tableWidth }}>
          <div
            className="grid w-full border-b border-slate-800 bg-slate-950 text-left text-xs uppercase tracking-wide text-slate-400"
            style={{ gridTemplateColumns }}
          >
            {table.getHeaderGroups().map((headerGroup) =>
              headerGroup.headers.map((header) => (
                <div key={header.id} className="relative px-3 py-2">
                  <span className="truncate">
                    {header.isPlaceholder
                      ? null
                      : flexRender(header.column.columnDef.header, header.getContext())}
                  </span>
                  <button
                    type="button"
                    aria-label={`Resize ${String(header.column.columnDef.header)} column`}
                    className="absolute inset-y-0 right-0 w-2 cursor-col-resize touch-none rounded bg-transparent hover:bg-slate-700/60 focus:bg-slate-700/60 focus:outline-none"
                    onMouseDown={(event) => {
                      markColumnAsResized(header.column.id as ColumnKey);
                      header.getResizeHandler()(event);
                    }}
                    onTouchStart={(event) => {
                      markColumnAsResized(header.column.id as ColumnKey);
                      header.getResizeHandler()(event);
                    }}
                  />
                </div>
              ))
            )}
          </div>

          <div
            ref={parentRef}
            className="min-h-0 flex-1 overflow-y-scroll overflow-x-hidden [scrollbar-gutter:stable]"
          >
            <div className="relative" style={{ height: `${rowVirtualizer.getTotalSize()}px` }}>
              {rowVirtualizer.getVirtualItems().map((virtualRow) => {
                const row = table.getRowModel().rows[virtualRow.index];
                const sourceRecord = rows[virtualRow.index];
                const metadata = metadataByFileId[sourceRecord.id];
                const moveResult = moveResultsByFileId[sourceRecord.id];
                const isSkippedUnsupported = sourceRecord.scan_status === "skipped_unsupported";
                const isFallbackWarning = metadata?.date_kind === "fallback";
                const isMoved = moveResult?.result_status === "moved";
                const isMoveError =
                  moveResult?.result_status === "skipped" || moveResult?.result_status === "failed";

                return (
                  <div
                    key={row.id}
                    className={`absolute left-0 top-0 grid w-full border-t border-slate-800 text-sm ${isMoved ? "bg-emerald-950/30 text-emerald-200" : ""} ${isMoveError || isSkippedUnsupported ? "bg-rose-950/30 text-rose-200" : ""} ${isFallbackWarning ? "bg-amber-950/20" : ""}`}
                    style={{
                      gridTemplateColumns,
                      transform: `translateY(${virtualRow.start}px)`,
                      height: `${virtualRow.size}px`
                    }}
                  >
                    {row.getVisibleCells().map((cell) => (
                      <div key={cell.id}>
                        {flexRender(cell.column.columnDef.cell, cell.getContext())}
                      </div>
                    ))}
                  </div>
                );
              })}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function resolveColumnData(
  columnKey: ColumnKey,
  row: FileRecord,
  metadata: MetadataResult | undefined,
  movePlan: MovePlan | undefined,
  moveResult: MoveResult | undefined,
  selectedFolder: string | null
): { value: string; title: string; className: string } {
  const metadataDateLabel = metadata
    ? (metadata.chosen_date ?? metadata.raw_metadata_date ?? "No metadata date")
    : "pending";
  const destinationLabel =
    movePlan?.destination_path || (metadata?.metadata_status === "ready" ? "planning..." : "-");
  const relativeDestinationLabel = resolveRelativePath(destinationLabel, selectedFolder);
  const statusLabel = resolveStatusLabel(row, metadata, movePlan, moveResult);
  const notesLabel = resolveNotesLabel(metadata, movePlan, moveResult);

  switch (columnKey) {
    case "source":
      return {
        value: row.relative_path,
        title: row.relative_path,
        className: "truncate px-3 py-2 font-mono text-xs"
      };
    case "destination":
      return {
        value: relativeDestinationLabel,
        title: destinationLabel,
        className: "truncate px-3 py-2 font-mono text-xs text-slate-300"
      };
    case "status":
      return { value: statusLabel, title: statusLabel, className: "truncate px-3 py-2" };
    case "kind":
      return { value: row.kind, title: row.kind, className: "truncate px-3 py-2" };
    case "originalName":
      return {
        value: row.original_name,
        title: row.original_name,
        className: "truncate px-3 py-2"
      };
    case "metadataDate":
      return {
        value: metadataDateLabel,
        title: metadataDateLabel,
        className: "truncate px-3 py-2"
      };
    case "notes":
      return { value: notesLabel, title: notesLabel, className: "truncate px-3 py-2" };
  }
}

function resolveRelativePath(pathValue: string, basePath: string | null): string {
  if (!basePath || pathValue === "-" || pathValue === "planning...") {
    return pathValue;
  }

  const normalizedBase = normalizePath(basePath);
  const normalizedPath = normalizePath(pathValue);
  if (!normalizedPath.toLowerCase().startsWith(`${normalizedBase.toLowerCase()}/`)) {
    return pathValue;
  }

  return normalizedPath.slice(normalizedBase.length + 1);
}

function normalizePath(pathValue: string): string {
  return pathValue.replace(/\\/g, "/").replace(/\/+$/, "");
}

function resolveStatusLabel(
  row: FileRecord,
  metadata: MetadataResult | undefined,
  movePlan: MovePlan | undefined,
  moveResult: MoveResult | undefined
): string {
  if (moveResult) {
    switch (moveResult.result_status) {
      case "moved":
        return "Moved";
      case "skipped":
        return "Skipped";
      case "failed":
        return "Failed";
      default:
        return "Processed";
    }
  }

  if (row.scan_status !== "discovered") {
    return STATUS_LABELS[row.scan_status];
  }

  if (metadata?.metadata_status === "error") {
    return "Metadata Error";
  }
  if (metadata?.date_kind === "fallback") {
    return "Warning Fallback";
  }

  if (!movePlan) {
    return metadata?.metadata_status === "ready" ? "Planning" : "Pending Metadata";
  }

  switch (movePlan.plan_status) {
    case "ready":
      return movePlan.conflict_status === "suffix_applied" ? "Planned (Suffix)" : "Planned";
    case "blocked_no_date":
      return "Blocked No Date";
    case "blocked_conflict":
      return "Blocked Conflict";
    case "error":
      return "Plan Error";
    default:
      return "Pending Plan";
  }
}

function resolveNotesLabel(
  metadata: MetadataResult | undefined,
  movePlan: MovePlan | undefined,
  moveResult: MoveResult | undefined
): string {
  if (moveResult?.error) {
    return moveResult.error;
  }

  if (metadata?.metadata_status === "error") {
    return metadata.error ?? "Metadata error";
  }
  if (metadata?.date_kind === "fallback") {
    return "Using FileModifyDate fallback";
  }
  if (!metadata) {
    return "-";
  }

  if (movePlan?.plan_status === "blocked_no_date") {
    return "No usable metadata date";
  }
  if (movePlan?.conflict_status === "suffix_applied") {
    return "Suffix applied for conflict";
  }

  return resolveDateSourceLabel(metadata.chosen_date_source);
}

function resolveDateSourceLabel(source: string | null): string {
  if (!source) {
    return "-";
  }

  return DATE_SOURCE_LABELS[source] ?? source;
}
