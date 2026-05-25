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
  { key: "source", label: "Source" },
  { key: "destination", label: "Destination" },
  { key: "status", label: "Status" }
] as const;

type ColumnKey = (typeof COLUMNS)[number]["key"];

type TableRowData = {
  id: string;
  error: string;
} & Record<ColumnKey, string>;

const DEFAULT_COLUMN_SIZING: ColumnSizingState = {
  source: 300,
  destination: 300,
  status: 120
};

export function HistoryDetailsPanel({ details, loading, error }: HistoryDetailsPanelProps) {
  const tableRows = useMemo<TableRowData[]>(
    () =>
      details?.operations.map((operation) => ({
        id: operation.operation_id,
        status: formatOperationStatus(operation.status),
        source: operation.source_path,
        destination: operation.destination_path,
        error: operation.error ?? ""
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
          const isPath = column.key === "source" || column.key === "destination";
          const error = context.row.original.error;
          const title = column.key === "status" && error ? `${value}: ${error}` : value;
          return (
            <div
              className={`truncate px-2 py-1 ${isPath ? "font-mono text-slate-400" : ""} ${column.key === "status" && error ? "text-rose-400" : ""}`}
              title={title}
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
