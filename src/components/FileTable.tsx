import { useVirtualizer } from "@tanstack/react-virtual";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { PointerEvent as ReactPointerEvent } from "react";
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

const SOURCE_COLUMN_INDEX = 0;
const DESTINATION_COLUMN_INDEX = 1;
const COLUMN_HORIZONTAL_PADDING = 28;
const MONO_CELL_FONT = "12px ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace";
const BODY_CELL_FONT =
  "14px system-ui, -apple-system, Segoe UI, Roboto, Helvetica, Arial, sans-serif";

interface ActiveResize {
  columnIndex: number;
  startX: number;
  startWidth: number;
}

export function FileTable({
  rows,
  metadataByFileId,
  movePlansByFileId,
  moveResultsByFileId,
  selectedFolder
}: FileTableProps) {
  const parentRef = useRef<HTMLDivElement | null>(null);
  const activeResizeRef = useRef<ActiveResize | null>(null);
  const manuallyResizedColumnsRef = useRef<Set<number>>(new Set());
  const [columnWidths, setColumnWidths] = useState<number[]>(() =>
    COLUMNS.map(() => COLUMN_HORIZONTAL_PADDING)
  );
  const gridTemplateColumns = useMemo(
    () => columnWidths.map((width) => `${width}px`).join(" "),
    [columnWidths]
  );
  const tableMinWidth = useMemo(
    () => columnWidths.reduce((total, width) => total + width, 0),
    [columnWidths]
  );

  const rowVirtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 38,
    overscan: 20
  });

  const handleResizeStart = useCallback(
    (columnIndex: number) => (event: ReactPointerEvent<HTMLButtonElement>) => {
      event.preventDefault();
      activeResizeRef.current = {
        columnIndex,
        startX: event.clientX,
        startWidth: columnWidths[columnIndex]
      };
      manuallyResizedColumnsRef.current.add(columnIndex);
      document.body.classList.add("cursor-col-resize", "select-none");
    },
    [columnWidths]
  );

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

    const desiredWidths = COLUMNS.map((column, columnIndex) => {
      const headerWidth = measureWidth(column.label, BODY_CELL_FONT);
      const cellMaxWidth = rows.reduce((maxWidth, row) => {
        const metadata = metadataByFileId[row.id];
        const movePlan = movePlansByFileId[row.id];
        const moveResult = moveResultsByFileId[row.id];

        const metadataDateLabel = metadata
          ? (metadata.chosen_date ?? metadata.raw_metadata_date ?? "No metadata date")
          : "pending";
        const destinationLabel =
          movePlan?.destination_path ||
          (metadata?.metadata_status === "ready" ? "planning..." : "-");
        const relativeDestinationLabel = resolveRelativePath(destinationLabel, selectedFolder);
        const statusLabel = resolveStatusLabel(row, metadata, movePlan, moveResult);
        const notesLabel = resolveNotesLabel(metadata, movePlan, moveResult);

        const value = (() => {
          switch (columnIndex) {
            case SOURCE_COLUMN_INDEX:
              return row.relative_path;
            case DESTINATION_COLUMN_INDEX:
              return relativeDestinationLabel;
            case 2:
              return statusLabel;
            case 3:
              return row.kind;
            case 4:
              return row.original_name;
            case 5:
              return metadataDateLabel;
            case 6:
              return notesLabel;
            default:
              return "";
          }
        })();

        const font =
          columnIndex === SOURCE_COLUMN_INDEX || columnIndex === DESTINATION_COLUMN_INDEX
            ? MONO_CELL_FONT
            : BODY_CELL_FONT;
        return Math.max(maxWidth, measureWidth(value, font));
      }, 0);

      return Math.max(
        headerWidth + COLUMN_HORIZONTAL_PADDING,
        cellMaxWidth + COLUMN_HORIZONTAL_PADDING
      );
    });

    setColumnWidths((previous) => {
      let changed = false;
      const next = [...previous];

      for (let columnIndex = 0; columnIndex < COLUMNS.length; columnIndex += 1) {
        if (manuallyResizedColumnsRef.current.has(columnIndex)) {
          continue;
        }

        const desiredWidth = desiredWidths[columnIndex];
        if (next[columnIndex] !== desiredWidth) {
          next[columnIndex] = desiredWidth;
          changed = true;
        }
      }

      return changed ? next : previous;
    });
  }, [rows, metadataByFileId, movePlansByFileId, moveResultsByFileId, selectedFolder]);

  useEffect(() => {
    const handlePointerMove = (event: PointerEvent) => {
      const activeResize = activeResizeRef.current;
      if (!activeResize) {
        return;
      }

      const nextWidth = Math.max(
        0,
        activeResize.startWidth + (event.clientX - activeResize.startX)
      );

      setColumnWidths((previous) => {
        if (previous[activeResize.columnIndex] === nextWidth) {
          return previous;
        }

        const next = [...previous];
        next[activeResize.columnIndex] = nextWidth;
        return next;
      });
    };

    const stopResizing = () => {
      if (!activeResizeRef.current) {
        return;
      }

      activeResizeRef.current = null;
      document.body.classList.remove("cursor-col-resize", "select-none");
    };

    window.addEventListener("pointermove", handlePointerMove);
    window.addEventListener("pointerup", stopResizing);

    return () => {
      window.removeEventListener("pointermove", handlePointerMove);
      window.removeEventListener("pointerup", stopResizing);
      document.body.classList.remove("cursor-col-resize", "select-none");
    };
  }, []);

  return (
    <div className="flex h-full flex-col overflow-hidden">
      <div className="min-h-0 flex-1 overflow-x-auto">
        <div className="flex h-full flex-col" style={{ minWidth: `${tableMinWidth}px` }}>
          <div
            className="grid border-b border-slate-800 bg-slate-950 text-left text-xs uppercase tracking-wide text-slate-400"
            style={{ gridTemplateColumns }}
          >
            {COLUMNS.map((column, columnIndex) => (
              <div key={column.key} className="relative px-3 py-2">
                <span className="truncate">{column.label}</span>
                <button
                  type="button"
                  aria-label={`Resize ${column.label} column`}
                  className="absolute inset-y-0 right-0 w-2 cursor-col-resize touch-none rounded bg-transparent hover:bg-slate-700/60 focus:bg-slate-700/60 focus:outline-none"
                  onPointerDown={handleResizeStart(columnIndex)}
                />
              </div>
            ))}
          </div>

          <div
            ref={parentRef}
            className="min-h-0 flex-1 overflow-y-scroll overflow-x-hidden [scrollbar-gutter:stable]"
          >
            <div className="relative" style={{ height: `${rowVirtualizer.getTotalSize()}px` }}>
              {rowVirtualizer.getVirtualItems().map((virtualRow) => {
                const row = rows[virtualRow.index];
                const isSkippedUnsupported = row.scan_status === "skipped_unsupported";
                const metadata = metadataByFileId[row.id];
                const movePlan = movePlansByFileId[row.id];
                const moveResult = moveResultsByFileId[row.id];
                const metadataDateLabel = metadata
                  ? (metadata.chosen_date ?? metadata.raw_metadata_date ?? "No metadata date")
                  : "pending";
                const destinationLabel =
                  movePlan?.destination_path ||
                  (metadata?.metadata_status === "ready" ? "planning..." : "-");
                const relativeDestinationLabel = resolveRelativePath(
                  destinationLabel,
                  selectedFolder
                );

                const statusLabel = resolveStatusLabel(row, metadata, movePlan, moveResult);
                const notesLabel = resolveNotesLabel(metadata, movePlan, moveResult);
                const isFallbackWarning = metadata?.date_kind === "fallback";
                const isMoved = moveResult?.result_status === "moved";
                const isMoveError =
                  moveResult?.result_status === "skipped" || moveResult?.result_status === "failed";

                return (
                  <div
                    key={row.id}
                    className={`absolute left-0 top-0 grid border-t border-slate-800 text-sm ${isMoved ? "bg-emerald-950/30 text-emerald-200" : ""} ${isMoveError || isSkippedUnsupported ? "bg-rose-950/30 text-rose-200" : ""} ${isFallbackWarning ? "bg-amber-950/20" : ""}`}
                    style={{
                      gridTemplateColumns,
                      transform: `translateY(${virtualRow.start}px)`,
                      height: `${virtualRow.size}px`
                    }}
                  >
                    <div className="px-3 py-2 font-mono text-xs" title={row.relative_path}>
                      {row.relative_path}
                    </div>
                    <div
                      className="px-3 py-2 font-mono text-xs text-slate-300"
                      title={destinationLabel}
                    >
                      {relativeDestinationLabel}
                    </div>
                    <div className="truncate px-3 py-2" title={statusLabel}>
                      {statusLabel}
                    </div>
                    <div className="truncate px-3 py-2" title={row.kind}>
                      {row.kind}
                    </div>
                    <div className="truncate px-3 py-2" title={row.original_name}>
                      {row.original_name}
                    </div>
                    <div className="truncate px-3 py-2" title={metadataDateLabel}>
                      {metadataDateLabel}
                    </div>
                    <div className="truncate px-3 py-2" title={notesLabel}>
                      {notesLabel}
                    </div>
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
