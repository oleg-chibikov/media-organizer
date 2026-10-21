import type { MoveExecutionState } from "../moves/moveExecutionState";
import type { ScanProgress } from "../scan/scanProgress";

export interface PlanningStatus {
  isPlanning: boolean;
  planCompleted: boolean;
  planErrors: number;
}

export function pipelineStatusLabel(
  scan: ScanProgress,
  planning: PlanningStatus,
  execution: MoveExecutionState
): string {
  if (scan.discovered === 0) {
    return "No files";
  }
  if (scan.cancelled) {
    return "Scan cancelled";
  }
  if (execution.paused) {
    return "Dispatch paused";
  }
  if (execution.executing) {
    return "Executing moves";
  }
  if (execution.completed) {
    return "Move completed";
  }
  if ((scan.completed && !planning.planCompleted) || planning.isPlanning) {
    return "Planning destinations";
  }
  return scan.completed ? "Destinations ready" : "Idle";
}

export function executionActionLabel(execution: MoveExecutionState): string {
  if (execution.completed) {
    return "Complete";
  }
  if (!execution.confirmed) {
    return "Confirm & Execute";
  }
  return execution.paused ? "Resume Dispatch" : "Pause Dispatch";
}
