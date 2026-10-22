import type { MoveConfirmationSummary } from "../../lib/types";
import { hasMoveResults, type MoveCounts } from "../moves";
import type { MetadataProgress } from "../metadata";
import type { ScanProgress } from "../scan";

interface StatusCountsProps {
  scan: ScanProgress;
  dates: MetadataProgress;
  planErrors: number;
  summary: MoveConfirmationSummary;
  showReady: boolean;
  moveCounts: MoveCounts;
}

export function StatusCounts({
  scan,
  dates,
  planErrors,
  summary,
  showReady,
  moveCounts
}: StatusCountsProps) {
  return (
    <>
      <span>Files: {scan.discovered}</span>
      {dates.read < dates.total ? (
        <span>
          Dates read: {dates.read}/{dates.total}
        </span>
      ) : null}
      {scan.skipped > 0 ? <span>Skipped: {scan.skipped}</span> : null}
      {scan.errors > 0 ? <span className="text-amber-300">Unreadable: {scan.errors}</span> : null}
      {planErrors > 0 ? <span className="text-amber-300">Can't plan: {planErrors}</span> : null}
      {summary.total_plans > 0 ? (
        <>
          {showReady ? <span>Ready: {summary.ready_operations}</span> : null}
          {summary.suffix_applied > 0 ? <span>Renamed: {summary.suffix_applied}</span> : null}
        </>
      ) : null}
      {hasMoveResults(moveCounts) ? (
        <>
          {moveCounts.moved > 0 ? <span>Moved: {moveCounts.moved}</span> : null}
          {moveCounts.copied > 0 ? <span>Copied: {moveCounts.copied}</span> : null}
          {moveCounts.skipped > 0 ? <span>Not moved: {moveCounts.skipped}</span> : null}
          {moveCounts.failed > 0 ? (
            <span className="text-amber-300">Failed: {moveCounts.failed}</span>
          ) : null}
        </>
      ) : null}
    </>
  );
}
