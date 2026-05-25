import { type ColumnDef, type ColumnSizingState } from "@tanstack/react-table";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { ReactNode } from "react";
import { revealFileInExplorer } from "../lib/api";
import type { HistoryIndexEntry, HistoryRunDetails, HistoryRunStatus } from "../lib/types";
import { ResizableVirtualTable } from "./ResizableVirtualTable";

interface ToolbarProps {
  selectedFolder: string | null;
  folderError: string | null;
  isScanning: boolean;
  historyRuns: HistoryIndexEntry[];
  selectedHistoryRunId: string | null;
  historyLoading: boolean;
  details: HistoryRunDetails | null;
  detailsLoading: boolean;
  detailsError: string | null;
  settingsMenu?: ReactNode;
  onChooseFolder: () => void;
  onCancelScan: () => void;
  onSelectHistoryRun: (runId: string) => void;
  onUndoProcessing: (runId: string) => void;
  onRedoProcessing: (runId: string) => void;
  onDeleteProcessing: (runId: string) => void;
}

const RECENT_RUN_COLUMNS = [
  { key: "source", label: "Source Path" },
  { key: "destination", label: "Destination Path" },
  { key: "migrated", label: "Files Migrated" },
  { key: "date", label: "Date / Time" },
  { key: "status", label: "Status" },
  { key: "actions", label: "Actions" }
] as const;

type RecentRunColumnKey = (typeof RECENT_RUN_COLUMNS)[number]["key"];
type RecentRunTableRow = { id: string; run: HistoryIndexEntry } & Record<
  RecentRunColumnKey,
  string
>;

const RECENT_RUN_DEFAULT_SIZING: ColumnSizingState = {
  source: 240,
  destination: 240,
  migrated: 110,
  date: 170,
  status: 140,
  actions: 210
};

const DETAILS_COLUMNS = [
  { key: "source", label: "Source" },
  { key: "destination", label: "Destination" },
  { key: "status", label: "Status" }
] as const;

type DetailsColumnKey = (typeof DETAILS_COLUMNS)[number]["key"];
type DetailsTableRow = { id: string; error: string } & Record<DetailsColumnKey, string>;

const DETAILS_DEFAULT_SIZING: ColumnSizingState = {
  source: 300,
  destination: 300,
  status: 120
};

export function Toolbar({
  selectedFolder,
  folderError,
  isScanning,
  historyRuns,
  selectedHistoryRunId,
  historyLoading,
  details,
  detailsLoading,
  detailsError,
  settingsMenu,
  onChooseFolder,
  onCancelScan,
  onSelectHistoryRun,
  onUndoProcessing,
  onRedoProcessing,
  onDeleteProcessing
}: ToolbarProps) {
  const [historyOpen, setHistoryOpen] = useState(false);
  const historyRef = useRef<HTMLDivElement>(null);

  const handleClickOutside = useCallback((event: MouseEvent) => {
    if (historyRef.current && !historyRef.current.contains(event.target as Node)) {
      setHistoryOpen(false);
    }
  }, []);

  useEffect(() => {
    if (historyOpen) {
      document.addEventListener("mousedown", handleClickOutside);
    }
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [historyOpen, handleClickOutside]);

  const recentRunRows = useMemo<RecentRunTableRow[]>(
    () =>
      historyRuns.map((run) => ({
        id: run.run_id,
        run,
        source: run.selected_folder,
        destination:
          (details?.run.run_id === run.run_id ? details.run.destination_root_path : null) ??
          `${run.selected_folder}\\Processed`,
        migrated:
          details?.run.run_id === run.run_id
            ? String(
                details.operations.filter(
                  (operation) =>
                    operation.operation_kind === "process" && operation.status === "succeeded"
                ).length
              )
            : "-",
        date: formatHistoryTimestamp(run.started_at),
        status: formatRunStatus(run.run_status),
        actions: "actions"
      })),
    [details, historyRuns]
  );

  const recentRunColumnDefs = useMemo<ColumnDef<RecentRunTableRow>[]>(
    () =>
      RECENT_RUN_COLUMNS.map((column) => ({
        id: column.key,
        header: column.label,
        accessorKey: column.key,
        cell: (context) => {
          const row = context.row.original;
          if (column.key === "actions") {
            const run = row.run;
            return (
              <div
                className="flex justify-start gap-1 px-2 py-1"
                onClick={(event) => event.stopPropagation()}
              >
                {run.undo_available ? (
                  <button
                    aria-label={`Undo ${run.folder_label}`}
                    className="rounded bg-slate-800 px-2 py-1 text-[11px] font-medium text-slate-100 hover:bg-slate-700"
                    onClick={() => onUndoProcessing(run.run_id)}
                    type="button"
                  >
                    Undo
                  </button>
                ) : run.redo_available ? (
                  <button
                    aria-label={`Redo ${run.folder_label}`}
                    className="rounded bg-slate-800 px-2 py-1 text-[11px] font-medium text-slate-100 hover:bg-slate-700"
                    onClick={() => onRedoProcessing(run.run_id)}
                    type="button"
                  >
                    Redo
                  </button>
                ) : null}
                <button
                  aria-label={`Open log for ${run.folder_label}`}
                  className="rounded bg-slate-800 px-2 py-1 text-[11px] font-medium text-slate-100 hover:bg-slate-700"
                  onClick={() => revealFileInExplorer(run.history_file_path).catch(() => {})}
                  type="button"
                >
                  Log
                </button>
                <button
                  aria-label={`Delete ${run.folder_label}`}
                  className="rounded bg-slate-800 px-2 py-1 text-[11px] font-medium text-red-400 hover:bg-red-900 hover:text-red-300"
                  onClick={() => onDeleteProcessing(run.run_id)}
                  type="button"
                >
                  x
                </button>
              </div>
            );
          }

          const value = String(context.getValue() ?? "");
          const mono = column.key === "date" || column.key === "migrated";
          const muted = column.key === "status";
          return (
            <button
              aria-label={`View run ${row.run.run_id}`}
              className={`w-full truncate px-2 py-1 text-left text-xs ${mono ? "font-mono text-[11px]" : ""} ${muted ? "text-[11px] text-slate-400" : ""}`}
              onClick={(event) => {
                event.stopPropagation();
                onSelectHistoryRun(row.run.run_id);
              }}
              title={value}
              type="button"
            >
              {value}
            </button>
          );
        }
      })),
    [onDeleteProcessing, onRedoProcessing, onSelectHistoryRun, onUndoProcessing]
  );

  const detailsRows = useMemo<DetailsTableRow[]>(
    () =>
      details?.operations.map((operation) => {
        const sourceRoot = details.run.selected_folder;
        const destinationRoot =
          details.run.destination_root_path ?? `${details.run.selected_folder}\\Processed`;
        return {
          id: operation.operation_id,
          status: formatOperationStatus(operation.status),
          source: toRelativePath(operation.source_path, sourceRoot),
          destination: toRelativePath(operation.destination_path, destinationRoot),
          error: operation.error ?? ""
        };
      }) ?? [],
    [details]
  );

  const detailsColumnDefs = useMemo<ColumnDef<DetailsTableRow>[]>(
    () =>
      DETAILS_COLUMNS.map((column) => ({
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
    <header className="flex flex-wrap items-center gap-2 border-b border-slate-800 bg-slate-900 px-3 py-2">
      <button
        className="rounded bg-sky-600 px-3 py-1.5 text-sm font-medium text-white hover:bg-sky-500 disabled:cursor-not-allowed disabled:bg-slate-700"
        onClick={onChooseFolder}
        type="button"
      >
        Choose Folder
      </button>
      {isScanning ? (
        <button
          className="rounded bg-rose-600 px-3 py-1.5 text-sm font-medium text-white hover:bg-rose-500 disabled:cursor-not-allowed disabled:bg-slate-700"
          onClick={onCancelScan}
          type="button"
        >
          Stop Scan
        </button>
      ) : null}

      <div ref={historyRef} className="relative">
        <button
          className="cursor-pointer rounded bg-slate-800 px-3 py-1.5 text-sm font-medium text-slate-100 hover:bg-slate-700"
          onClick={() => setHistoryOpen((prev) => !prev)}
          type="button"
        >
          History
        </button>
        {historyOpen && (
          <div className="absolute left-0 z-20 mt-1 w-[48rem] rounded border border-slate-700 bg-slate-900 p-2 shadow-lg">
            <p className="mb-1 text-xs text-slate-400">Recent Runs</p>
            <div className="max-h-56 overflow-y-auto">
              {historyLoading ? (
                <p className="px-2 py-1 text-xs text-slate-500">Loading history...</p>
              ) : historyRuns.length === 0 ? (
                <p className="px-2 py-1 text-xs text-slate-500">No history runs</p>
              ) : (
                <ResizableVirtualTable
                  rows={recentRunRows}
                  columns={RECENT_RUN_COLUMNS}
                  columnDefs={recentRunColumnDefs}
                  defaultColumnSizing={RECENT_RUN_DEFAULT_SIZING}
                  onAutoSizeColumns={() => RECENT_RUN_DEFAULT_SIZING}
                  selectable={true}
                  onRowClick={(row) => onSelectHistoryRun(row.run.run_id)}
                  virtualized={false}
                  getRowClassName={(row) =>
                    selectedHistoryRunId === row.run.run_id
                      ? "bg-sky-800 text-sky-100 hover:bg-sky-700"
                      : "text-slate-300 hover:bg-slate-800"
                  }
                />
              )}
            </div>

            {selectedHistoryRunId && (details || detailsLoading || detailsError) ? (
              <div className="mt-2 border-t border-slate-700 pt-2">
                <div className="mb-2 flex items-center gap-2">
                  <span className="text-xs font-semibold text-slate-200">History Details</span>
                  {details ? (
                    <>
                      <span className="font-mono text-[10px] text-slate-500">
                        {details.run.run_id}
                      </span>
                      <span className="text-xs text-slate-300">{details.run.folder_label}</span>
                    </>
                  ) : null}
                  {details?.unsafe_for_undo_redo ? (
                    <span className="rounded bg-amber-900/40 px-1.5 py-0.5 text-[10px] text-amber-300">
                      Unsafe for Undo/Redo
                    </span>
                  ) : null}
                </div>
                {detailsError ? (
                  <p className="mb-1 text-[11px] text-rose-400">{detailsError}</p>
                ) : null}
                {detailsLoading ? (
                  <p className="text-[11px] text-slate-500">Loading history details...</p>
                ) : details ? (
                  details.operations.length > 0 ? (
                    <div className="max-h-40 overflow-auto border border-slate-800">
                      <ResizableVirtualTable
                        rows={detailsRows}
                        columns={DETAILS_COLUMNS}
                        columnDefs={detailsColumnDefs}
                        defaultColumnSizing={DETAILS_DEFAULT_SIZING}
                        onAutoSizeColumns={() => DETAILS_DEFAULT_SIZING}
                        virtualized={false}
                      />
                    </div>
                  ) : (
                    <p className="text-[11px] text-slate-500">
                      No history operations recorded for this run.
                    </p>
                  )
                ) : null}
              </div>
            ) : null}
          </div>
        )}
      </div>

      <div className="ml-2 min-w-0 flex-1 text-sm text-slate-300">
        {selectedFolder ? (
          <p className="truncate">{selectedFolder}</p>
        ) : (
          <p className="text-slate-500">No folder selected</p>
        )}
        {folderError ? <p className="text-rose-400">{folderError}</p> : null}
      </div>

      {settingsMenu ? <div className="ml-auto shrink-0">{settingsMenu}</div> : null}
    </header>
  );
}

function toRelativePath(path: string, root: string): string {
  const normalizedPath = path.replace(/\//g, "\\").toLowerCase();
  const normalizedRoot = root.replace(/\//g, "\\").toLowerCase();
  if (normalizedPath === normalizedRoot) {
    return ".";
  }
  const rootedPrefix = normalizedRoot.endsWith("\\") ? normalizedRoot : `${normalizedRoot}\\`;
  if (!normalizedPath.startsWith(rootedPrefix)) {
    return path;
  }
  return path.slice(rootedPrefix.length);
}

function formatRunStatus(status: HistoryRunStatus): string {
  switch (status) {
    case "completed":
      return "Completed";
    case "completed_with_errors":
      return "Completed With Errors";
    case "running":
      return "Running";
    case "undone":
      return "Undone";
    case "partially_undone":
      return "Partially Undone";
    case "redone":
      return "Redone";
    case "partial":
      return "Partial";
    case "corrupt":
      return "Corrupt";
    default:
      return status;
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

function formatHistoryTimestamp(timestamp: string): string {
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
  return `${dateStr}  ${timeStr}`;
}
