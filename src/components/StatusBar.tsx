import { Button } from "./Button";
import type { MoveConfirmationSummary } from "../lib/types";

interface StatusBarProps {
  discovered: number;
  skipped: number;
  errors: number;
  planErrors: number;
  confirmationSummary: MoveConfirmationSummary;
  executionConfirmed: boolean;
  isPlanning: boolean;
  planCompleted: boolean;
  completed: boolean;
  cancelled: boolean;
  isExecutingMoves: boolean;
  isMoveDispatchPaused: boolean;
  moveCompleted: boolean;
  moved: number;
  copied: number;
  moveSkipped: number;
  moveFailed: number;
  hasPendingExecutableItems: boolean;
  onExecutionAction: () => void;
}

export function StatusBar({
  discovered,
  skipped,
  errors,
  planErrors,
  confirmationSummary,
  executionConfirmed,
  isPlanning,
  planCompleted,
  completed,
  cancelled,
  isExecutingMoves,
  isMoveDispatchPaused,
  moveCompleted,
  moved,
  copied,
  moveSkipped,
  moveFailed,
  hasPendingExecutableItems,
  onExecutionAction
}: StatusBarProps) {
  const statusLabel = cancelled
    ? "Scan cancelled"
    : isExecutingMoves
      ? "Executing moves"
      : moveCompleted
        ? "Move completed"
        : completed && !planCompleted
          ? "Planning destinations"
          : isPlanning
            ? "Planning destinations"
            : completed && planCompleted
              ? "Destinations planned"
              : completed
                ? "Scan completed"
                : "Idle";
  const plannedButNotReady =
    confirmationSummary.total_plans > 0
      ? confirmationSummary.total_plans - confirmationSummary.ready_operations
      : 0;
  const hasMoveResults = moved + copied + moveSkipped + moveFailed > 0;
  const actionLabel = moveCompleted
    ? "Complete"
    : !executionConfirmed
      ? "Confirm & Execute"
      : isMoveDispatchPaused
        ? "Resume Dispatch"
        : "Pause Dispatch";

  return (
    <footer className="border-t border-slate-800 bg-slate-900 px-3 py-2 text-xs text-slate-300">
      <div className="flex flex-wrap items-center gap-x-4 gap-y-2">
        <span className="font-semibold text-slate-100">{statusLabel}</span>
        <span>Files: {discovered}</span>
        {skipped > 0 ? <span>Scan skipped: {skipped}</span> : null}
        {errors > 0 ? <span className="text-amber-300">Scan errors: {errors}</span> : null}
        {planErrors > 0 ? <span className="text-amber-300">Plan errors: {planErrors}</span> : null}
        {confirmationSummary.total_plans > 0 ? (
          <>
            <span>Ready: {confirmationSummary.ready_operations}</span>
            {plannedButNotReady > 0 ? <span>Needs review: {plannedButNotReady}</span> : null}
            {confirmationSummary.fallback_warnings > 0 ? (
              <span>Fallback dates: {confirmationSummary.fallback_warnings}</span>
            ) : null}
            {confirmationSummary.suffix_applied > 0 ? (
              <span>Renamed conflicts: {confirmationSummary.suffix_applied}</span>
            ) : null}
          </>
        ) : (
          <span className="text-slate-500">No move summary yet</span>
        )}
        {hasMoveResults ? (
          <>
            {moved > 0 ? <span>Moved: {moved}</span> : null}
            {copied > 0 ? <span>Copied: {copied}</span> : null}
            {moveSkipped > 0 ? <span>Move skipped: {moveSkipped}</span> : null}
            {moveFailed > 0 ? (
              <span className="text-amber-300">Move failed: {moveFailed}</span>
            ) : null}
          </>
        ) : null}
        <Button
          variant="primary"
          className="ml-auto px-3 py-1.5 text-xs"
          disabled={moveCompleted || !hasPendingExecutableItems}
          onClick={onExecutionAction}
        >
          {actionLabel}
        </Button>
      </div>
      {executionConfirmed && !moveCompleted ? (
        <p className={`mt-1 ${isMoveDispatchPaused ? "text-amber-400" : "text-emerald-400"}`}>
          {isMoveDispatchPaused
            ? "Move dispatch paused. In-flight jobs may still complete."
            : "Execution explicitly confirmed."}
        </p>
      ) : null}
    </footer>
  );
}
