import { type ColumnDef, type ColumnSizingState } from "@tanstack/react-table";
import { useMemo } from "react";
import type { HistoryRunDetails } from "../lib/types";
import { ResizableVirtualTable } from "./ResizableVirtualTable";

interface HistoryDetailsPanelProps {
  details: HistoryRunDetails | null;
  loading: boolean;
  error: string | null;
}

const COLUMNS = [
  { key: "time", label: "Time" },
  { key: "kind", label: "Kind" },
  { key: "file", label: "File" },
  { key: "size", label: "Size" },
  { key: "status", label: "Status" },
  { key: "source", label: "Source" },
  { key: "destination", label: "Destination" },
  { key: "error", label: "Error" }
] as const;

type ColumnKey = (typeof COLUMNS)[number]["key"];

type TableRowData = {
  id: string;
} & Record<ColumnKey, string>;

const DEFAULT_COLUMN_SIZING: ColumnSizingState = {
  time: 170,
  kind: 80,
  file: 200,
  size: 90,
  status: 90,
  source: 220,
  destination: 220,
  error: 220
};

export function HistoryDetailsPanel({ details, loading, error }: HistoryDetailsPanelProps) {
  const tableRows = useMemo<TableRowData[]>(
    () =>
      details?.operations.map((operation) => ({
        id: operation.operation_id,
        time: formatHistoryTime(operation.timestamp),
        kind: formatOperationKind(operation.operation_kind),
        file: operation.original_name,
        size: formatFileSize(operation.expected_destination_size ?? operation.expected_source_size),
        status: formatOperationStatus(operation.status),
        source: operation.source_path,
        destination: operation.destination_path,
        error: operation.error ?? "-"
      })) ?? [],
    [details]
  );

  const tableColumns = useMemo<ColumnDef<TableRowData>[]>(
    () =>
      COLUMNS.map((column) => ({
        id: column.key,
        header: column.label,
        accessorKey: column.key,
        cell: (context) => {
          const value = String(context.getValue() ?? "");
          const isNarrow = column.key === "time" || column.key === "size";
          const isPath = column.key === "source" || column.key === "destination";
          const isError = column.key === "error";
          return (
            <div
              className={`truncate px-2 py-1 ${isNarrow ? "font-mono text-slate-400" : ""} ${isPath ? "font-mono text-slate-400" : ""} ${isError ? "text-rose-400" : ""}`}
              title={value}
            >
              {value}
            </div>
          );
        }
      })),
    []
  );

  return (
    <section className="border-t border-slate-800 bg-slate-950 px-3 py-2 text-xs text-slate-300">
      <div className="mb-2 flex items-center gap-2">
        <span className="font-semibold text-slate-200">History Details</span>
        {details ? (
          <>
            <span className="font-mono text-slate-500">{details.run.run_id}</span>
            <span>{details.run.folder_label}</span>
          </>
        ) : null}
        {details?.unsafe_for_undo_redo ? (
          <span className="rounded bg-amber-900/40 px-2 py-0.5 text-amber-300">
            Unsafe for Undo/Redo
          </span>
        ) : null}
      </div>
      {error ? <p className="mb-2 text-rose-400">{error}</p> : null}
      {loading ? (
        <p className="text-slate-500">Loading history details...</p>
      ) : details ? (
        details.operations.length > 0 ? (
          <div className="max-h-40 overflow-auto border border-slate-800">
            <ResizableVirtualTable
              rows={tableRows}
              columns={COLUMNS}
              columnDefs={tableColumns}
              defaultColumnSizing={DEFAULT_COLUMN_SIZING}
              onAutoSizeColumns={() => DEFAULT_COLUMN_SIZING}
              virtualized={false}
            />
          </div>
        ) : (
          <p className="text-slate-500">No history operations recorded for this run.</p>
        )
      ) : (
        <p className="text-slate-500">Select a run from the History menu to inspect details.</p>
      )}
    </section>
  );
}

function formatHistoryTime(timestamp: string): string {
  const date = new Date(timestamp);
  if (Number.isNaN(date.getTime())) {
    return timestamp;
  }
  const dateStr = date.toLocaleDateString("en-US", {
    year: "numeric",
    month: "short",
    day: "numeric"
  });
  const timeStr = date.toLocaleTimeString("en-US", {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hour12: false
  });
  return `${dateStr}\u00A0\u00A0${timeStr}`;
}

function formatFileSize(bytes: number | null): string {
  if (bytes === null || bytes === undefined) return "-";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

function formatOperationKind(kind: string): string {
  switch (kind) {
    case "process":
      return "Process";
    case "undo":
      return "Undo";
    case "redo":
      return "Redo";
    default:
      return kind;
  }
}

function formatOperationStatus(status: string): string {
  switch (status) {
    case "succeeded":
      return "Succeeded";
    case "skipped":
      return "Skipped";
    case "blocked":
      return "Blocked";
    case "failed":
      return "Failed";
    default:
      return status;
  }
}
