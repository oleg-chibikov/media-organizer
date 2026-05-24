import type { HistoryIndexEntry, HistoryRunStatus } from "../lib/types";

interface ToolbarProps {
  selectedFolder: string | null;
  folderError: string | null;
  isScanning: boolean;
  historyRuns: HistoryIndexEntry[];
  selectedHistoryRunId: string | null;
  historyLoading: boolean;
  undoEnabled: boolean;
  redoEnabled: boolean;
  onChooseFolder: () => void;
  onCancelScan: () => void;
  onSelectHistoryRun: (runId: string) => void;
  onUndoProcessing: () => void;
  onRedoProcessing: () => void;
}

export function Toolbar({
  selectedFolder,
  folderError,
  isScanning,
  historyRuns,
  selectedHistoryRunId,
  historyLoading,
  undoEnabled,
  redoEnabled,
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
        <div className="absolute left-0 z-20 mt-1 w-[28rem] rounded border border-slate-700 bg-slate-900 p-2 shadow-lg">
          <div className="mb-2 grid grid-cols-2 gap-2">
            <button
              className="rounded bg-slate-800 px-2 py-1.5 text-left text-xs font-medium text-slate-100 hover:bg-slate-700 disabled:cursor-not-allowed disabled:text-slate-500"
              disabled={!undoEnabled}
              onClick={onUndoProcessing}
              type="button"
            >
              Undo Processing
            </button>
            <button
              className="rounded bg-slate-800 px-2 py-1.5 text-left text-xs font-medium text-slate-100 hover:bg-slate-700 disabled:cursor-not-allowed disabled:text-slate-500"
              disabled={!redoEnabled}
              onClick={onRedoProcessing}
              type="button"
            >
              Redo Processing
            </button>
          </div>
          <p className="mb-1 text-xs text-slate-400">Recent Runs</p>
          <div className="max-h-56 overflow-y-auto">
            {historyLoading ? (
              <p className="px-2 py-1 text-xs text-slate-500">Loading history...</p>
            ) : historyRuns.length === 0 ? (
              <p className="px-2 py-1 text-xs text-slate-500">No history runs</p>
            ) : (
              historyRuns.map((run) => (
                <button
                  key={run.run_id}
                  className={`mb-1 flex w-full items-center gap-2 rounded px-2 py-1 text-left text-xs hover:bg-slate-800 ${selectedHistoryRunId === run.run_id ? "bg-slate-800 text-slate-100" : "text-slate-300"}`}
                  onClick={() => onSelectHistoryRun(run.run_id)}
                  type="button"
                >
                  <span className="w-40 shrink-0 font-mono text-[11px]">
                    {formatHistoryTimestamp(run.started_at)}
                  </span>
                  <span className="min-w-0 flex-1 truncate">{run.folder_label}</span>
                  <span className="shrink-0 text-[11px] text-slate-400">
                    {formatRunStatus(run.run_status)}
                  </span>
                </button>
              ))
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

  return date.toLocaleString();
}
