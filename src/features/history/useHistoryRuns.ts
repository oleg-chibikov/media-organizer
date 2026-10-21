import { useCallback, useEffect, useState } from "react";
import { listHistoryRuns } from "../../lib/api";
import type { HistoryIndexEntry } from "../../lib/types";
import { resolveSelectedRunId } from "./resolveSelectedRunId";

export function useHistoryRuns(setHistoryError: (message: string | null) => void) {
  const [runs, setRuns] = useState<HistoryIndexEntry[]>([]);
  const [selectedRunId, setSelectedRunId] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const reloadRuns = useCallback(
    async (preferredRunId: string | null = null) => {
      setLoading(true);
      try {
        const nextRuns = await listHistoryRuns();
        setRuns(nextRuns);
        setHistoryError(null);
        setSelectedRunId((previous) => resolveSelectedRunId(previous, preferredRunId, nextRuns));
      } catch (error) {
        setHistoryError(`Failed to load history runs: ${String(error)}`);
      } finally {
        setLoading(false);
      }
    },
    [setHistoryError]
  );

  const dropRun = useCallback((runId: string) => {
    setRuns((previous) => previous.filter((run) => run.run_id !== runId));
    setSelectedRunId((previous) => (previous === runId ? null : previous));
  }, []);

  useEffect(() => {
    queueMicrotask(() => {
      void reloadRuns();
    });
  }, [reloadRuns]);

  return { runs, selectedRunId, setSelectedRunId, loading, reloadRuns, dropRun };
}
