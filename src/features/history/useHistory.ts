import { useRef, useState } from "react";
import { attachHistoryListeners, useBackendEvents } from "../../lib/api";
import {
  advanceHistoryAction,
  type ActiveHistoryAction,
  type HistoryActionKind
} from "./historyActionProgress";
import { useHistoryActions } from "./useHistoryActions";
import { useHistoryRunDetails } from "./useHistoryRunDetails";
import { useHistoryRuns } from "./useHistoryRuns";

export function useHistory(reportError: (message: string | null) => void) {
  const [historyError, setHistoryError] = useState<string | null>(null);
  const { runs, selectedRunId, setSelectedRunId, loading, reloadRuns } =
    useHistoryRuns(setHistoryError);
  const details = useHistoryRunDetails(selectedRunId, setHistoryError);
  const activeActionRef = useRef<ActiveHistoryAction | null>(null);
  const actions = useHistoryActions({
    activeActionRef,
    reloadRuns,
    setDetails: details.setDetails,
    setSelectedRunId,
    setHistoryError,
    reportError
  });

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
      onIndexUpdated: ({ run_id }) => void reloadRuns(run_id).then(details.reload),
      onUndoProgress: (payload) => trackProgress("undo", payload),
      onRedoProgress: (payload) => trackProgress("redo", payload)
    },
    (error) => setHistoryError(`Failed to attach history listeners: ${String(error)}`)
  );

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
    ...actions
  };
}
