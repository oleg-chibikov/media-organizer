import { useVirtualizer } from "@tanstack/react-virtual";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { PointerEvent as ReactPointerEvent } from "react";
import type { FileRecord, MetadataResult, MovePlan } from "../lib/types";

interface FileTableProps {
  rows: FileRecord[];
  metadataByFileId: Record<string, MetadataResult>;
  movePlansByFileId: Record<string, MovePlan>;
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
  { key: "status", label: "Status", defaultWidth: 224, minWidth: 140 },
  { key: "source", label: "Source File", defaultWidth: 320, minWidth: 220 },
  { key: "kind", label: "Kind", defaultWidth: 128, minWidth: 96 },
  { key: "originalName", label: "Original Name", defaultWidth: 192, minWidth: 140 },
  { key: "metadataDate", label: "Metadata Date", defaultWidth: 176, minWidth: 140 },
  { key: "destination", label: "Destination", defaultWidth: 320, minWidth: 220 },
  { key: "notes", label: "Notes", defaultWidth: 224, minWidth: 160 }
] as const;

interface ActiveResize {
  columnIndex: number;
  startX: number;
  startWidth: number;
}

export function FileTable({ rows, metadataByFileId, movePlansByFileId }: FileTableProps) {
  const parentRef = useRef<HTMLDivElement | null>(null);
  const activeResizeRef = useRef<ActiveResize | null>(null);
  const [columnWidths, setColumnWidths] = useState<number[]>(() =>
    COLUMNS.map((column) => column.defaultWidth)
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
      document.body.classList.add("cursor-col-resize", "select-none");
    },
    [columnWidths]
  );

  useEffect(() => {
    const handlePointerMove = (event: PointerEvent) => {
      const activeResize = activeResizeRef.current;
      if (!activeResize) {
        return;
      }

      const column = COLUMNS[activeResize.columnIndex];
      const nextWidth = Math.max(
        column.minWidth,
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
    <div className="flex h-full flex-col">
      <div
        className="grid border-b border-slate-800 bg-slate-950 text-left text-xs uppercase tracking-wide text-slate-400"
        style={{ gridTemplateColumns, minWidth: `${tableMinWidth}px` }}
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

      <div ref={parentRef} className="min-h-0 flex-1 overflow-auto">
        <div
          className="relative"
          style={{ height: `${rowVirtualizer.getTotalSize()}px`, minWidth: `${tableMinWidth}px` }}
        >
          {rowVirtualizer.getVirtualItems().map((virtualRow) => {
            const row = rows[virtualRow.index];
            const isSkippedUnsupported = row.scan_status === "skipped_unsupported";
            const metadata = metadataByFileId[row.id];
            const movePlan = movePlansByFileId[row.id];
            const metadataDateLabel = metadata
              ? (metadata.chosen_date ?? metadata.raw_metadata_date ?? "No metadata date")
              : "pending";
            const destinationLabel =
              movePlan?.destination_path ||
              (metadata?.metadata_status === "ready" ? "planning..." : "-");

            const statusLabel = resolveStatusLabel(row, metadata, movePlan);
            const notesLabel = resolveNotesLabel(metadata, movePlan);
            const isFallbackWarning = metadata?.date_kind === "fallback";

            return (
              <div
                key={row.id}
                className={`absolute left-0 top-0 grid border-t border-slate-800 text-sm ${isSkippedUnsupported ? "bg-rose-950/30 text-rose-200" : ""} ${isFallbackWarning ? "bg-amber-950/20" : ""}`}
                style={{
                  gridTemplateColumns,
                  transform: `translateY(${virtualRow.start}px)`,
                  height: `${virtualRow.size}px`,
                  minWidth: `${tableMinWidth}px`
                }}
              >
                <div className="truncate px-3 py-2" title={statusLabel}>
                  {statusLabel}
                </div>
                <div className="truncate px-3 py-2 font-mono text-xs" title={row.relative_path}>
                  {row.relative_path}
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
                <div
                  className="truncate px-3 py-2 font-mono text-xs text-slate-300"
                  title={destinationLabel}
                >
                  {destinationLabel}
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
  );
}

function resolveStatusLabel(
  row: FileRecord,
  metadata: MetadataResult | undefined,
  movePlan: MovePlan | undefined
): string {
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
  movePlan: MovePlan | undefined
): string {
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
