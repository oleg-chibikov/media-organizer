import type { MoveConfirmationSummary } from "../../lib/types";
import { hasMoveResults, type MoveCounts } from "../moves";
import type { ScanProgress } from "../scan";

interface StatusCountsProps {
  scan: ScanProgress;
  planErrors: number;
  summary: MoveConfirmationSummary;
  moveCounts: MoveCounts;
}

export function StatusCounts({ scan, planErrors, summary, moveCounts }: StatusCountsProps) {
  const needsReview = summary.total_plans > 0 ? summary.total_plans - summary.ready_operations : 0;

  return (
    <>
      <span>Files: {scan.discovered}</span>
      {scan.skipped > 0 ? <span>Scan skipped: {scan.skipped}</span> : null}
      {scan.errors > 0 ? <span className="text-amber-300">Scan errors: {scan.errors}</span> : null}
      {planErrors > 0 ? <span className="text-amber-300">Plan errors: {planErrors}</span> : null}
      {summary.total_plans > 0 ? (
        <>
          <span>Ready: {summary.ready_operations}</span>
          {needsReview > 0 ? <span>Needs review: {needsReview}</span> : null}
          {summary.fallback_warnings > 0 ? (
            <span>Fallback dates: {summary.fallback_warnings}</span>
          ) : null}
          {summary.suffix_applied > 0 ? (
            <span>Renamed conflicts: {summary.suffix_applied}</span>
          ) : null}
        </>
      ) : (
        <span className="text-slate-500">No move summary yet</span>
      )}
      {hasMoveResults(moveCounts) ? (
        <>
          {moveCounts.moved > 0 ? <span>Moved: {moveCounts.moved}</span> : null}
          {moveCounts.copied > 0 ? <span>Copied: {moveCounts.copied}</span> : null}
          {moveCounts.skipped > 0 ? <span>Move skipped: {moveCounts.skipped}</span> : null}
          {moveCounts.failed > 0 ? (
            <span className="text-amber-300">Move failed: {moveCounts.failed}</span>
          ) : null}
        </>
      ) : null}
    </>
  );
}
