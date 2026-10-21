import type { HistoryRedoProgressEvent, HistoryUndoProgressEvent } from "../../lib/types";

export type HistoryActionKind = "undo" | "redo";

export interface ActiveHistoryAction {
  runId: string;
  action: HistoryActionKind;
  blockedOrFailed: number;
}

export interface HistoryActionProgress {
  active: ActiveHistoryAction | null;
  failureMessage: string | null;
}

export function advanceHistoryAction(
  active: ActiveHistoryAction | null,
  action: HistoryActionKind,
  payload: HistoryUndoProgressEvent | HistoryRedoProgressEvent
): HistoryActionProgress {
  if (!active || active.action !== action || active.runId !== payload.run_id) {
    return { active, failureMessage: null };
  }
  const status = payload.operation.status;
  const blockedOrFailed =
    active.blockedOrFailed + (status === "blocked" || status === "failed" ? 1 : 0);
  if (payload.completed_operations < payload.total_operations) {
    return { active: { ...active, blockedOrFailed }, failureMessage: null };
  }
  const label = action === "undo" ? "Undo" : "Redo";
  return {
    active: null,
    failureMessage:
      blockedOrFailed > 0
        ? `${label} could not be completed: ${blockedOrFailed} operation(s) were blocked or failed.`
        : null
  };
}
