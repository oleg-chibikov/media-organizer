import { useMemo } from "react";
import type { HistoryRunDetails } from "../../lib/types";
import { HistoryOperationsTable } from "./HistoryOperationsTable";
import { buildOperationRows } from "./operationRows";

interface HistoryRunDetailsSectionProps {
  details: HistoryRunDetails | null;
  loading: boolean;
  error: string | null;
}

export function HistoryRunDetailsSection({
  details,
  loading,
  error
}: HistoryRunDetailsSectionProps) {
  const rows = useMemo(() => (details ? buildOperationRows(details, "relative") : []), [details]);

  return (
    <div className="mt-2 border-t border-slate-700 pt-2">
      <div className="mb-2 flex items-center gap-2">
        <span className="text-xs font-semibold text-slate-200">History Details</span>
        {details ? (
          <>
            <span className="font-mono text-[10px] text-slate-500">{details.run.run_id}</span>
            <span className="text-xs text-slate-300">{details.run.folder_label}</span>
          </>
        ) : null}
        {details?.unsafe_for_undo_redo ? (
          <span className="rounded bg-amber-900/40 px-1.5 py-0.5 text-[10px] text-amber-300">
            Unsafe for Undo/Redo
          </span>
        ) : null}
      </div>
      {error ? <p className="mb-1 text-[11px] text-rose-400">{error}</p> : null}
      {loading ? (
        <p className="text-[11px] text-slate-500">Loading history details...</p>
      ) : details ? (
        rows.length > 0 ? (
          <HistoryOperationsTable rows={rows} />
        ) : (
          <p className="text-[11px] text-slate-500">No history operations recorded for this run.</p>
        )
      ) : null}
    </div>
  );
}
