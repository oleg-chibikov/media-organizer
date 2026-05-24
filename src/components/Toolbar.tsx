import { revealFileInExplorer } from "../lib/api";
import type { HistoryIndexEntry, HistoryRunStatus } from "../lib/types";

interface ToolbarProps {
  selectedFolder: string | null;
  folderError: string | null;
  isScanning: boolean;
  historyRuns: HistoryIndexEntry[];
  selectedHistoryRunId: string | null;
  historyLoading: boolean;
  onChooseFolder: () => void;
  onCancelScan: () => void;
  onSelectHistoryRun: (runId: string) => void;
  onUndoProcessing: (runId: string) => void;
  onRedoProcessing: (runId: string) => void;
}

export function Toolbar({
  selectedFolder,
  folderError,
  isScanning,
  historyRuns,
  selectedHistoryRunId,
  historyLoading,
  onChooseFolder,
  onCancelScan,
  onSelectHistoryRun,
  onUndoProcessing,
  onRedoProcessing
}: ToolbarProps) {
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

      <details className="relative">
        <summary className="cursor-pointer list-none rounded bg-slate-800 px-3 py-1.5 text-sm font-medium text-slate-100 hover:bg-slate-700">
          History
        </summary>
        <div className="absolute left-0 z-20 mt-1 w-[32rem] rounded border border-slate-700 bg-slate-900 p-2 shadow-lg">
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
                  <span className="w-20 shrink-0">Status</span>
                  <span className="w-16 shrink-0 text-right">Actions</span>
                </div>
                {historyRuns.map((run) => (
                  <div
                    key={run.run_id}
                    className={`mb-1 flex items-center gap-2 rounded px-2 py-1 text-xs hover:bg-slate-800 ${selectedHistoryRunId === run.run_id ? "bg-sky-800 text-sky-100" : "text-slate-300"}`}
                  >
                    <button
                      aria-label={`View ${run.folder_label}`}
                      className="flex min-w-0 flex-1 cursor-pointer items-center gap-2 text-left"
                      onClick={() => onSelectHistoryRun(run.run_id)}
                      type="button"
                    >
                      <span className="w-44 shrink-0 font-mono text-[11px]">
                        {formatHistoryTimestamp(run.started_at)}
                      </span>
                      <span className="min-w-0 flex-1 truncate">{run.folder_label}</span>
                      <span className="w-20 shrink-0 text-[11px] text-slate-400">
                        {formatRunStatus(run.run_status)}
                      </span>
                    </button>
                    <div className="flex w-16 shrink-0 justify-end gap-1">
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
                    </div>
                  </div>
                ))}
              </>
            )}
          </div>
        </div>
      </details>

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
