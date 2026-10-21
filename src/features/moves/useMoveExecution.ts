import { useCallback, useMemo, type RefObject } from "react";
import type { FileRecord, MetadataResult, MovePlan } from "../../lib/types";
import { buildMoveConfirmationPrompt, countPendingExecutableMoves } from "./moveConfirmationPrompt";
import { IDLE_MOVE_EXECUTION, type MoveExecutionState } from "./moveExecutionState";
import { summarizeMoveConfirmation } from "./summarizeMoveConfirmation";
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
  const { selectedFolder, rows, metadataByFileId, movePlansByFileId, reportError } = options;
  const status = useMoveStatus();
  const { execution, moveResultsByFileId, updateExecution, clearStatus } = status;
  const dispatch = useMoveDispatch({
    ...options,
    ...status,
    executionConfirmed: execution.confirmed
  });
  const { resetDispatch, startDispatch, togglePaused, pumpRef } = dispatch;
  useMoveEvents({ ...options, ...dispatch, status });

  const confirmationSummary = useMemo(
    () => summarizeMoveConfirmation(rows, metadataByFileId, movePlansByFileId),
    [metadataByFileId, movePlansByFileId, rows]
  );
  const pendingExecutableCount = countPendingExecutableMoves(
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

  const confirmExecution = () => {
    if (pendingExecutableCount === 0 || execution.confirmed) {
      return;
    }
    if (!window.confirm(buildMoveConfirmationPrompt(confirmationSummary))) {
      return;
    }
    if (!selectedFolder) {
      reportError("No selected folder for move execution.");
      return;
    }
    const statusMessage = "Dispatching move execution.";
    clearMoves(() => ({ ...IDLE_MOVE_EXECUTION, confirmed: true, executing: true, statusMessage }));
    startDispatch();
    pumpRef.current();
  };

  const togglePause = () => {
    if (!execution.confirmed || execution.completed) {
      return;
    }
    const paused = togglePaused();
    updateExecution({
      paused,
      statusMessage: paused ? "Move dispatch paused by user." : "Move dispatch resumed."
    });
  };

  return {
    execution,
    moveResultsByFileId,
    moveCounts: status.moveCounts,
    confirmationSummary,
    hasPendingExecutableItems: pendingExecutableCount > 0,
    runExecutionAction: execution.confirmed ? togglePause : confirmExecution,
    resetMoves,
    resetMovesForReplan
  };
}
