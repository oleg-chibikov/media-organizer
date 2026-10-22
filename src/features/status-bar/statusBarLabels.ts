import type { MoveExecutionState } from "../moves";
import type { ScanProgress } from "../scan";

export interface PlanningStatus {
  isPlanning: boolean;
  planCompleted: boolean;
  planErrors: number;
}

export function pipelineStatusLabel(
  scan: ScanProgress,
  planning: PlanningStatus,
  execution: MoveExecutionState,
  readyOperations: number
): string {
  if (scan.discovered === 0) {
    return "No files";
  }
  if (scan.cancelled) {
    return "Scan stopped";
  }
  if (execution.paused) {
    return "Paused";
  }
  if (execution.executing) {
    return "Moving files";
  }
  if (execution.completed) {
    return "Done";
  }
  if ((scan.completed && !planning.planCompleted) || planning.isPlanning) {
    return "Checking files";
  }
  if (!scan.completed) {
    return "Scanning";
  }
  // A folder sorted earlier plans every file in place, so no file is left to move.
  return readyOperations > 0 ? "Ready to move" : "Nothing to move";
}

export function executionActionLabel(execution: MoveExecutionState): string {
  if (!execution.confirmed) {
    return "Move Files...";
  }
  return execution.paused ? "Resume" : "Pause";
}
