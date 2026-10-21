import { Button } from "../../components/design-system/Button";
import type { MoveConfirmationSummary } from "../../lib/types";
import type { MoveCounts, MoveExecutionState } from "../moves";
import type { ScanProgress } from "../scan";
import { executionActionLabel, pipelineStatusLabel, type PlanningStatus } from "./statusBarLabels";
import { StatusCounts } from "./StatusCounts";

interface StatusBarProps {
  health: string;
  scan: ScanProgress;
  planning: PlanningStatus;
  summary: MoveConfirmationSummary;
  execution: MoveExecutionState;
  moveCounts: MoveCounts;
  hasPendingExecutableItems: boolean;
  onExecutionAction: () => void;
}

export function StatusBar({
  health,
  scan,
  planning,
  summary,
  execution,
  moveCounts,
  hasPendingExecutableItems,
  onExecutionAction
}: StatusBarProps) {
  return (
    <footer className="border-t border-slate-800 bg-slate-900 px-3 py-2 text-xs text-slate-300">
      <div className="flex flex-wrap items-center gap-x-4 gap-y-2">
        <span role="status" className="font-semibold text-slate-100">
          {pipelineStatusLabel(scan, planning, execution)}
        </span>
        <StatusCounts
          scan={scan}
          planErrors={planning.planErrors}
          summary={summary}
          moveCounts={moveCounts}
        />
        <span className="text-slate-400">Backend: {health}</span>
        <Button
          variant="primary"
          className="ml-auto px-3 py-1.5 text-xs"
          disabled={execution.completed || !hasPendingExecutableItems}
          onClick={onExecutionAction}
        >
          {executionActionLabel(execution)}
        </Button>
      </div>
      {execution.confirmed && !execution.completed ? (
        <p className={`mt-1 ${execution.paused ? "text-amber-400" : "text-emerald-400"}`}>
          {execution.paused
            ? "Move dispatch paused. In-flight jobs may still complete."
            : (execution.statusMessage ?? "Execution started. Waiting for backend progress.")}
        </p>
      ) : null}
    </footer>
  );
}
