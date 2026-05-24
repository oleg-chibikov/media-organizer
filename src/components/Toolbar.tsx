import { useCallback, useEffect, useRef, useState } from "react";
import type { ReactNode } from "react";
import { revealFileInExplorer } from "../lib/api";
import type { HistoryIndexEntry, HistoryRunDetails, HistoryRunStatus } from "../lib/types";

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

  return (
    <header className="flex flex-wrap items-center gap-2 border-b border-slate-800 bg-slate-900 px-3 py-2">
      <button
        className="rounded bg-sky-600 px-3 py-1.5 text-sm font-medium text-white hover:bg-sky-500 disabled:cursor-not-allowed disabled:bg-slate-700"
        onClick={onChooseFolder}
        type="button"
      >
        Choose Folder
      </button>
      {settingsMenu}
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
                <>
                  <div className="flex items-center gap-2 px-2 py-1 text-[10px] font-semibold uppercase tracking-wider text-slate-500">
                    <span className="w-44 shrink-0">Date / Time</span>
                    <span className="min-w-0 flex-1">Folder</span>
                    <span className="w-28 shrink-0">Status</span>
                    <span className="w-24 shrink-0 text-right">Actions</span>
                  </div>
                  {historyRuns.map((run) => (
                    <div
                      key={run.run_id}
                      className={`mb-1 flex items-center gap-2 rounded px-2 py-1 text-xs ${selectedHistoryRunId === run.run_id ? "bg-sky-800 text-sky-100 hover:bg-sky-700" : "text-slate-300 hover:bg-slate-800"}`}
                    >
                      <button
                        aria-label={`View ${run.folder_label}`}
                        className="flex min-w-0 flex-1 cursor-pointer items-center gap-2 text-left"
                        onClick={() => {
                          onSelectHistoryRun(run.run_id);
                        }}
                        type="button"
                      >
                        <span className="w-44 shrink-0 font-mono text-[11px]">
                          {formatHistoryTimestamp(run.started_at)}
                        </span>
                        <span className="min-w-0 flex-1 truncate">{run.folder_label}</span>
                        <span className="w-28 shrink-0 text-[11px] text-slate-400">
                          {formatRunStatus(run.run_status)}
                        </span>
                      </button>
                      <div className="flex w-24 shrink-0 justify-end gap-1">
                        {run.undo_available ? (
                          <button
                            aria-label={`Undo ${run.folder_label}`}
                            className="rounded bg-slate-800 px-2 py-1 text-[11px] font-medium text-slate-100 hover:bg-slate-700"
                            onClick={() => {
                              onUndoProcessing(run.run_id);
                            }}
                            type="button"
                          >
                            Undo
                          </button>
                        ) : run.redo_available ? (
                          <button
                            aria-label={`Redo ${run.folder_label}`}
                            className="rounded bg-slate-800 px-2 py-1 text-[11px] font-medium text-slate-100 hover:bg-slate-700"
                            onClick={() => {
                              onRedoProcessing(run.run_id);
                            }}
                            type="button"
                          >
                            Redo
                          </button>
                        ) : null}
                        <button
                          aria-label={`Open log for ${run.folder_label}`}
                          className="rounded bg-slate-800 px-2 py-1 text-[11px] font-medium text-slate-100 hover:bg-slate-700"
                          onClick={() =>
                            revealFileInExplorer(run.history_file_path).catch(() => {})
                          }
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
                          ✕
                        </button>
                      </div>
                    </div>
                  ))}
                </>
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
                      <table className="w-full border-collapse text-left text-[11px]">
                        <thead>
                          <tr className="bg-slate-900 text-slate-400">
                            <th className="px-2 py-1">Time</th>
                            <th className="px-2 py-1">Kind</th>
                            <th className="px-2 py-1">File</th>
                            <th className="px-2 py-1">Size</th>
                            <th className="px-2 py-1">Status</th>
                            <th className="px-2 py-1">Source</th>
                            <th className="px-2 py-1">Destination</th>
                            <th className="px-2 py-1">Error</th>
                          </tr>
                        </thead>
                        <tbody>
                          {details.operations.map((operation) => (
                            <tr key={operation.operation_id} className="border-t border-slate-800">
                              <td className="px-2 py-1 font-mono">
                                {formatHistoryTime(operation.timestamp)}
                              </td>
                              <td className="px-2 py-1">
                                {formatOperationKind(operation.operation_kind)}
                              </td>
                              <td className="px-2 py-1">{operation.original_name}</td>
                              <td className="px-2 py-1 font-mono text-slate-400">
                                {formatFileSize(
                                  operation.expected_destination_size ??
                                    operation.expected_source_size
                                )}
                              </td>
                              <td className="px-2 py-1">
                                {formatOperationStatus(operation.status)}
                              </td>
                              <td className="max-w-[12rem] truncate px-2 py-1 font-mono text-slate-400">
                                {operation.source_path}
                              </td>
                              <td className="max-w-[12rem] truncate px-2 py-1 font-mono text-slate-400">
                                {operation.destination_path}
                              </td>
                              <td className="max-w-[14rem] truncate px-2 py-1 text-rose-400">
                                {operation.error ?? "-"}
                              </td>
                            </tr>
                          ))}
                        </tbody>
                      </table>
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
    </header>
  );
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
