import type { HistoryRunDetails } from "../lib/types";

interface HistoryDetailsPanelProps {
  details: HistoryRunDetails | null;
  loading: boolean;
  error: string | null;
}

export function HistoryDetailsPanel({ details, loading, error }: HistoryDetailsPanelProps) {
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
            <table className="w-full border-collapse text-left text-[11px]">
              <thead>
                <tr className="bg-slate-900 text-slate-400">
                  <th className="px-2 py-1">Time</th>
                  <th className="px-2 py-1">Kind</th>
                  <th className="px-2 py-1">File</th>
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
                    <td className="px-2 py-1">{formatOperationKind(operation.operation_kind)}</td>
                    <td className="px-2 py-1">{operation.original_name}</td>
                    <td className="px-2 py-1">{formatOperationStatus(operation.status)}</td>
                    <td className="max-w-[14rem] truncate px-2 py-1 font-mono text-slate-400">
                      {operation.source_path}
                    </td>
                    <td className="max-w-[14rem] truncate px-2 py-1 font-mono text-slate-400">
                      {operation.destination_path}
                    </td>
                    <td className="max-w-[16rem] truncate px-2 py-1 text-rose-400">
                      {operation.error ?? "-"}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
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
  return date.toLocaleString();
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
