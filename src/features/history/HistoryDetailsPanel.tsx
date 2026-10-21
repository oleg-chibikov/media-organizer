import { useMemo } from "react";
import type { HistoryRunDetails } from "../../lib/types";
import { HistoryOperationsTable } from "./HistoryOperationsTable";
import { buildOperationRows } from "./operationRows";

interface HistoryDetailsPanelProps {
  details: HistoryRunDetails | null;
  loading: boolean;
  error: string | null;
}

export function HistoryDetailsPanel({ details, loading, error }: HistoryDetailsPanelProps) {
  const rows = useMemo(() => (details ? buildOperationRows(details, "absolute") : []), [details]);

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
        rows.length > 0 ? (
          <HistoryOperationsTable rows={rows} />
        ) : (
          <p className="text-slate-500">No history operations recorded for this run.</p>
        )
      ) : (
        <p className="text-slate-500">Select a run from the History menu to inspect details.</p>
      )}
    </section>
  );
}
