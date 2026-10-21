import { useRef, useState } from "react";
import {
  attachHistoryListeners,
  deleteHistoryRun,
  getHistoryRun,
  redoHistoryRun,
  undoHistoryRun,
  useBackendEvents
} from "../../lib/api";
import {
  advanceHistoryAction,
  type ActiveHistoryAction,
  type HistoryActionKind
} from "./historyActionProgress";
import { confirmHistoryAction } from "./historyActionPrompt";
import { useHistoryRunDetails } from "./useHistoryRunDetails";
import { useHistoryRuns } from "./useHistoryRuns";

const RUN_HISTORY_ACTION = { undo: undoHistoryRun, redo: redoHistoryRun };

export function useHistory(reportError: (message: string | null) => void) {
  const [historyError, setHistoryError] = useState<string | null>(null);
  const { runs, selectedRunId, setSelectedRunId, loading, reloadRuns } =
    useHistoryRuns(setHistoryError);
  const details = useHistoryRunDetails(selectedRunId, setHistoryError);
  const activeActionRef = useRef<ActiveHistoryAction | null>(null);

  const trackProgress = (
    action: HistoryActionKind,
    payload: Parameters<typeof advanceHistoryAction>[2]
  ) => {
    const progress = advanceHistoryAction(activeActionRef.current, action, payload);
    activeActionRef.current = progress.active;
    if (progress.failureMessage) {
      reportError(progress.failureMessage);
    }
  };

  useBackendEvents(
    attachHistoryListeners,
    {
      onIndexUpdated: ({ run_id }) => void reloadRuns(run_id),
      onUndoProgress: (payload) => trackProgress("undo", payload),
      onRedoProgress: (payload) => trackProgress("redo", payload)
    },
    (error) => setHistoryError(`Failed to attach history listeners: ${String(error)}`)
  );

  const runAction = async (action: HistoryActionKind, runId: string) => {
    if (!confirmHistoryAction(action)) {
      return;
    }
    try {
      activeActionRef.current = { runId, action, blockedOrFailed: 0 };
      await RUN_HISTORY_ACTION[action](runId);
      reportError(null);
      await reloadRuns(runId);
      details.setDetails(await getHistoryRun(runId));
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
      details.setDetails((previous) => (previous?.run.run_id === runId ? null : previous));
      setSelectedRunId((previous) => (previous === runId ? null : previous));
      reportError(null);
      await reloadRuns();
    } catch (error) {
      reportError(`Failed to delete history entry: ${String(error)}`);
    }
  };

  const selectRun = (runId: string) => {
    if (runId !== selectedRunId) {
      setSelectedRunId(runId);
      details.setDetails(null);
    }
  };

  const visibleDetails =
    selectedRunId && details.details?.run.run_id === selectedRunId ? details.details : null;

  return {
    runs,
    selectedRunId,
    loading,
    details: visibleDetails,
    detailsLoading: details.loading,
    error: historyError,
    selectRun,
    undoRun: (runId: string) => void runAction("undo", runId),
    redoRun: (runId: string) => void runAction("redo", runId),
    deleteRun: (runId: string) => void deleteRun(runId)
  };
}
