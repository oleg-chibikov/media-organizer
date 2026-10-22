import type { FileIdMap } from "../../lib/fileIdMap";
import { useCallback, type RefObject } from "react";
import type { FileRecord, MetadataResult, MovePlan } from "../../lib/types";
import { IDLE_MOVE_EXECUTION, type MoveExecutionState } from "./moveExecutionState";
import { useMoveConfirmation } from "./useMoveConfirmation";
import { useMoveConfirmationSummary } from "./useMoveConfirmationSummary";
import { useMoveDispatch } from "./useMoveDispatch";
import { useMoveEvents } from "./useMoveEvents";
import { useMoveStatus } from "./useMoveStatus";
import { useSourceFolderCleanup } from "./useSourceFolderCleanup";

interface MoveExecutionOptions {
  // Checking files can still add ready plans, so a run may start before the first one.
  stillChecking: boolean;
  filesFound: boolean;
  selectedFolder: string | null;
  rows: FileRecord[];
  rowByIdRef: RefObject<Map<string, FileRecord>>;
  metadataByFileId: FileIdMap<MetadataResult>;
  movePlansByFileId: FileIdMap<MovePlan>;
  reportError: (message: string) => void;
}

export function useMoveExecution(options: MoveExecutionOptions) {
  const { stillChecking, filesFound, selectedFolder, rowByIdRef, reportError } = options;
  const { rows, metadataByFileId, movePlansByFileId } = options;
  const status = useMoveStatus();
  const { execution, moveResultsByFileId, updateExecution, clearStatus } = status;
  const { queueRef, pumpRef, resetDispatch, startDispatch, togglePaused } = useMoveDispatch({
    stillChecking,
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
  useSourceFolderCleanup(execution.completed, selectedFolder, reportError);

  const { confirmationSummary, pendingExecutableCount } = useMoveConfirmationSummary(
    rows,
    metadataByFileId,
    movePlansByFileId,
    moveResultsByFileId
  );

  const canStart = pendingExecutableCount > 0 || (stillChecking && filesFound);

  const clearMoves = useCallback(
    (nextExecution: (previous: MoveExecutionState) => MoveExecutionState) => {
      resetDispatch();
      clearStatus(nextExecution);
    },
    [clearStatus, resetDispatch]
  );
  const startExecution = () => {
    if (!canStart || execution.confirmed) {
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
  const confirmation = useMoveConfirmation({ ...options, onConfirmed: startExecution });

  const { cancel: cancelConfirmation } = confirmation;
  const resetMoves = useCallback(() => {
    cancelConfirmation();
    clearMoves(() => IDLE_MOVE_EXECUTION);
  }, [cancelConfirmation, clearMoves]);
  // Replanning keeps the executing flag because a running batch still reports back.
  const resetMovesForReplan = useCallback(
    () => clearMoves((previous) => ({ ...IDLE_MOVE_EXECUTION, executing: previous.executing })),
    [clearMoves]
  );

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
    canRunExecutionAction: execution.confirmed || canStart,
    confirmation: {
      summary: confirmation.summary,
      confirm: confirmation.confirm,
      cancel: confirmation.cancel
    },
    runExecutionAction: execution.confirmed ? togglePause : () => void confirmation.request(),
    resetMoves,
    resetMovesForReplan
  };
}
