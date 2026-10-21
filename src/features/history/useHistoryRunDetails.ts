import { useEffect, useState } from "react";
import { getHistoryRun } from "../../lib/api";
import type { HistoryRunDetails } from "../../lib/types";

export function useHistoryRunDetails(
  selectedRunId: string | null,
  setHistoryError: (message: string | null) => void
) {
  const [details, setDetails] = useState<HistoryRunDetails | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!selectedRunId) {
      return;
    }

    let ignore = false;
    queueMicrotask(() => {
      if (!ignore) {
        setLoading(true);
      }
    });
    void getHistoryRun(selectedRunId)
      .then((nextDetails) => {
        if (!ignore) {
          setDetails(nextDetails);
        }
      })
      .catch((error: unknown) => {
        if (!ignore) {
          setHistoryError(`Failed to load history details: ${String(error)}`);
        }
      })
      .finally(() => {
        if (!ignore) {
          setLoading(false);
        }
      });

    return () => {
      ignore = true;
    };
  }, [selectedRunId, setHistoryError]);

  return { details, setDetails, loading };
}
