import type { HistoryOperationStatus, HistoryRunStatus } from "../../lib/types";

const RUN_STATUS_LABELS: Record<HistoryRunStatus, string> = {
  completed: "Completed",
  completed_with_errors: "Completed With Errors",
  running: "Running",
  undone: "Undone",
  partially_undone: "Partially Undone",
  redone: "Redone",
  partial: "Partial",
  corrupt: "Corrupt"
};

const OPERATION_STATUS_LABELS: Record<HistoryOperationStatus, string> = {
  succeeded: "Succeeded",
  skipped: "Skipped",
  blocked: "Blocked",
  failed: "Failed"
};

export function formatRunStatus(status: HistoryRunStatus): string {
  return RUN_STATUS_LABELS[status] ?? status;
}

export function formatOperationStatus(status: HistoryOperationStatus): string {
  return OPERATION_STATUS_LABELS[status] ?? status;
}
