import { Button } from "../../components/design-system/Button";
import type { MoveConfirmationSummary } from "../../lib/types";
import type { MetadataProgress } from "../metadata";
import type { MoveCounts, MoveExecutionState } from "../moves";
import type { ScanProgress } from "../scan";
import { finishSummary } from "./finishSummary";
import { moveProgressCount } from "./moveProgressCount";
import { executionActionLabel, pipelineStatusLabel, type PlanningStatus } from "./statusBarLabels";
import { StatusCounts } from "./StatusCounts";

interface StatusBarProps {
  health: string;
  scan: ScanProgress;
  dates: MetadataProgress;
  planning: PlanningStatus;
  summary: MoveConfirmationSummary;
  execution: MoveExecutionState;
  moveCounts: MoveCounts;
  canRunExecutionAction: boolean;
  destinationRoot: string | null;
  onExecutionAction: () => void;
  onOpenDestination: () => void;
}

export function StatusBar({
  health,
  scan,
  dates,
  planning,
  summary,
  execution,
  moveCounts,
  canRunExecutionAction,
  destinationRoot,
  onExecutionAction,
  onOpenDestination
}: StatusBarProps) {
  return (
    <footer className="border-t border-slate-800 bg-slate-900 px-3 py-2 text-xs text-slate-300">
      <div className="flex flex-wrap items-center gap-x-4 gap-y-2">
        <span role="status" className="font-semibold text-slate-100">
          {pipelineStatusLabel(scan, planning, execution, summary.ready_operations)}
        </span>
        <StatusCounts
          scan={scan}
          dates={dates}
          planErrors={planning.planErrors}
          summary={summary}
          showReady={!execution.completed}
          moveCounts={moveCounts}
        />
        {health === "ok" || health === "checking" ? null : (
          <span className="text-rose-300">Backend: {health}</span>
        )}
        {execution.completed ? (
          <Button
            variant="primary"
            className="ml-auto px-3 py-1.5 text-xs"
            disabled={!destinationRoot}
            onClick={onOpenDestination}
          >
            Open Destination
          </Button>
        ) : (
          <Button
            variant="primary"
            className="ml-auto px-3 py-1.5 text-xs"
            disabled={!canRunExecutionAction}
            onClick={onExecutionAction}
          >
            {executionActionLabel(execution)}
          </Button>
        )}
      </div>
      {execution.completed ? (
        <p className="mt-1 text-emerald-300">{finishSummary(moveCounts)}</p>
      ) : null}
      {execution.confirmed && !execution.completed ? (
        <p className={`mt-1 ${execution.paused ? "text-amber-400" : "text-emerald-400"}`}>
          {moveProgressCount(scan.discovered, planning.planErrors, summary, moveCounts)}{" "}
          {execution.paused
            ? "Paused. The file moving now will finish."
            : (execution.statusMessage ?? "Starting...")}
        </p>
      ) : null}
    </footer>
  );
}
