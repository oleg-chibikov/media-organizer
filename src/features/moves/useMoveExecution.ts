import { useCallback, useMemo, useState, type RefObject } from "react";
import type { FileRecord, MetadataResult, MovePlan, MoveResult } from "../../lib/types";
import { EMPTY_MOVE_COUNTS } from "./moveCounts";
import { buildMoveConfirmationPrompt, countPendingExecutableMoves } from "./moveConfirmationPrompt";
import {
  IDLE_MOVE_EXECUTION,
  patchMoveExecution,
  type MoveExecutionState
} from "./moveExecutionState";
import { summarizeMoveConfirmation } from "./summarizeMoveConfirmation";
import { useMoveDispatch } from "./useMoveDispatch";
import { useMoveEvents } from "./useMoveEvents";

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
  const [execution, setExecution] = useState<MoveExecutionState>(IDLE_MOVE_EXECUTION);
  const [moveResultsByFileId, setMoveResultsByFileId] = useState<Record<string, MoveResult>>({});
  const [moveCounts, setMoveCounts] = useState(EMPTY_MOVE_COUNTS);
  const updateExecution = useCallback(
    (patch: Partial<MoveExecutionState>) =>
      setExecution((previous) => patchMoveExecution(previous, patch)),
    []
  );
  const dispatch = useMoveDispatch({
    ...options,
    moveResultsByFileId,
    executionConfirmed: execution.confirmed,
    updateExecution
  });
  useMoveEvents({ ...options, ...dispatch, setExecution, setMoveResultsByFileId, setMoveCounts });

  const confirmationSummary = useMemo(
    () => summarizeMoveConfirmation(rows, metadataByFileId, movePlansByFileId),
    [metadataByFileId, movePlansByFileId, rows]
  );
  const pendingExecutableCount = countPendingExecutableMoves(
    movePlansByFileId,
    moveResultsByFileId
  );
  const { resetDispatch, startDispatch, togglePaused, pumpRef } = dispatch;

  const clearMoves = useCallback(
    (nextExecution: (previous: MoveExecutionState) => MoveExecutionState) => {
      resetDispatch();
      setExecution(nextExecution);
      setMoveResultsByFileId({});
      setMoveCounts(EMPTY_MOVE_COUNTS);
    },
    [resetDispatch]
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
    clearMoves(() => ({
      ...IDLE_MOVE_EXECUTION,
      confirmed: true,
      executing: true,
      statusMessage: "Dispatching move execution."
    }));
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
    moveCounts,
    confirmationSummary,
    hasPendingExecutableItems: pendingExecutableCount > 0,
    runExecutionAction: execution.confirmed ? togglePause : confirmExecution,
    resetMoves,
    resetMovesForReplan
  };
}
