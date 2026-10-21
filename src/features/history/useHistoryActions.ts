import type { Dispatch, RefObject, SetStateAction } from "react";
import {
  deleteHistoryRun,
  getHistoryRun,
  redoHistoryRun,
  revealFileInExplorer,
  undoHistoryRun
} from "../../lib/api";
import type { HistoryRunDetails } from "../../lib/types";
import type { ActiveHistoryAction, HistoryActionKind } from "./historyActionProgress";
import { confirmHistoryAction } from "./historyActionPrompt";

const RUN_HISTORY_ACTION = { undo: undoHistoryRun, redo: redoHistoryRun };

interface HistoryActionsOptions {
  activeActionRef: RefObject<ActiveHistoryAction | null>;
  reloadRuns: (preferredRunId?: string | null) => Promise<void>;
  setDetails: Dispatch<SetStateAction<HistoryRunDetails | null>>;
  setSelectedRunId: Dispatch<SetStateAction<string | null>>;
  setHistoryError: (message: string | null) => void;
  reportError: (message: string | null) => void;
}

// Undo, redo and delete each ask first; the backend decides what happens to the files.
export function useHistoryActions(options: HistoryActionsOptions) {
  const {
    activeActionRef,
    reloadRuns,
    setDetails,
    setSelectedRunId,
    setHistoryError,
    reportError
  } = options;

  const runAction = async (action: HistoryActionKind, runId: string) => {
    if (!confirmHistoryAction(action)) {
      return;
    }
    try {
      activeActionRef.current = { runId, action, blockedOrFailed: 0 };
      await RUN_HISTORY_ACTION[action](runId);
      reportError(null);
      await reloadRuns(runId);
      setDetails(await getHistoryRun(runId));
    } catch (error) {
      activeActionRef.current = null;
      reportError(`Failed to ${action} processing: ${String(error)}`);
    }
  };

  const deleteRun = async (runId: string) => {
    if (!confirmHistoryAction("delete")) {
      return;
    }
    try {
      await deleteHistoryRun(runId);
      setDetails((previous) => (previous?.run.run_id === runId ? null : previous));
      setSelectedRunId((previous) => (previous === runId ? null : previous));
      reportError(null);
      await reloadRuns();
    } catch (error) {
      reportError(`Failed to delete history entry: ${String(error)}`);
    }
  };

  const openLog = (historyFilePath: string) =>
    void revealFileInExplorer(historyFilePath).catch((error: unknown) =>
      setHistoryError(`Failed to open the history log: ${String(error)}`)
    );

  return {
    undoRun: (runId: string) => void runAction("undo", runId),
    redoRun: (runId: string) => void runAction("redo", runId),
    deleteRun: (runId: string) => void deleteRun(runId),
    openLog
  };
}
