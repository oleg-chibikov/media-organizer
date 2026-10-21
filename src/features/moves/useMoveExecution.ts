import { useCallback, type RefObject } from "react";
import type { FileRecord, MetadataResult, MovePlan } from "../../lib/types";
import { IDLE_MOVE_EXECUTION, type MoveExecutionState } from "./moveExecutionState";
import { useMoveConfirmation } from "./useMoveConfirmation";
import { useMoveConfirmationSummary } from "./useMoveConfirmationSummary";
import { useMoveDispatch } from "./useMoveDispatch";
import { useMoveEvents } from "./useMoveEvents";
import { useMoveStatus } from "./useMoveStatus";

interface MoveExecutionOptions {
  selectedFolder: string | null;
  rows: FileRecord[];
  rowByIdRef: RefObject<Map<string, FileRecord>>;
  metadataByFileId: Record<string, MetadataResult>;
  movePlansByFileId: Record<string, MovePlan>;
  reportError: (message: string) => void;
}

export function useMoveExecution(options: MoveExecutionOptions) {
  const { selectedFolder, rows, rowByIdRef, metadataByFileId, movePlansByFileId, reportError } =
    options;
  const status = useMoveStatus();
  const { execution, moveResultsByFileId, updateExecution, clearStatus } = status;
  const { queueRef, pumpRef, resetDispatch, startDispatch, togglePaused } = useMoveDispatch({
    selectedFolder,
    rowByIdRef,
    metadataByFileId,
    movePlansByFileId,
    moveResultsByFileId,
    executionConfirmed: execution.confirmed,
    updateExecution,
    reportError
  });
  useMoveEvents({ queueRef, pumpRef, rowByIdRef, status, reportError });

  const { confirmationSummary, pendingExecutableCount } = useMoveConfirmationSummary(
    rows,
    metadataByFileId,
    movePlansByFileId,
    moveResultsByFileId
  );

  const clearMoves = useCallback(
    (nextExecution: (previous: MoveExecutionState) => MoveExecutionState) => {
      resetDispatch();
      clearStatus(nextExecution);
    },
    [clearStatus, resetDispatch]
  );
  const resetMoves = useCallback(() => clearMoves(() => IDLE_MOVE_EXECUTION), [clearMoves]);
  // Replanning keeps the executing flag because a running batch still reports back.
  const resetMovesForReplan = useCallback(
    () => clearMoves((previous) => ({ ...IDLE_MOVE_EXECUTION, executing: previous.executing })),
    [clearMoves]
  );

  const startExecution = () => {
    if (pendingExecutableCount === 0 || execution.confirmed) {
      return;
    }
    if (!selectedFolder) {
      reportError("No selected folder for move execution.");
      return;
    }
    const statusMessage = "Starting...";
    clearMoves(() => ({ ...IDLE_MOVE_EXECUTION, confirmed: true, executing: true, statusMessage }));
    startDispatch();
    pumpRef.current();
  };
  const confirmExecution = useMoveConfirmation({ ...options, onConfirmed: startExecution });

  const togglePause = () => {
    if (!execution.confirmed || execution.completed) {
      return;
    }
    const paused = togglePaused();
    updateExecution({
      paused,
      statusMessage: paused ? "Paused." : "Resumed."
    });
  };

  return {
    execution,
    moveResultsByFileId,
    moveCounts: status.moveCounts,
    confirmationSummary,
    hasPendingExecutableItems: pendingExecutableCount > 0,
    runExecutionAction: execution.confirmed ? togglePause : () => void confirmExecution(),
    resetMoves,
    resetMovesForReplan
  };
}
