import { useCallback, useRef, useState, type RefObject } from "react";
import { applyFilePlaces, clearScanPlans } from "../../lib/api";
import type { FilePlace } from "../../lib/types";
import { usePlanBatches } from "./usePlanBatches";
import { usePlanEvents } from "./usePlanEvents";
import { withoutFileIds } from "./withoutFileIds";

interface MovePlanningOptions {
  // The backend plans the files of this scan as their dates are read.
  scanIdRef: RefObject<number | null>;
  reportError: (message: string) => void;
}

// Shows the plans the backend makes, and tells it the place of each file.
export function useMovePlanning({ scanIdRef, reportError }: MovePlanningOptions) {
  const { movePlansByFileId, addPlan, flushPlans, dropPlans, dropPlansFor } = usePlanBatches();
  const [planErrorsByFileId, setPlanErrorsByFileId] = useState<Record<string, string>>({});
  // Plans wait for the place folders until the first apply, so no file shows a date folder first.
  const [waiting, setWaiting] = useState(true);
  const waitingRef = useRef(true);
  const [applying, setApplying] = useState(false);

  const wait = useCallback((nextWaiting: boolean) => {
    waitingRef.current = nextWaiting;
    setWaiting(nextWaiting);
  }, []);

  const forgetPlans = useCallback(() => {
    dropPlans();
    setPlanErrorsByFileId({});
  }, [dropPlans]);

  usePlanEvents({
    scanIdRef,
    addPlan: (plan) => {
      addPlan(plan);
      setPlanErrorsByFileId((previous) => withoutFileIds(previous, [plan.file_id]));
    },
    setError: (fileId, error) => {
      dropPlansFor([fileId]);
      setPlanErrorsByFileId((previous) => ({ ...previous, [fileId]: error }));
    },
    clearPlans: forgetPlans,
    onSettled: () => {
      // Plans land before planning counts as done.
      flushPlans();
      setApplying(false);
    },
    reportError
  });

  const applyPlaces = useCallback(
    (filePlaces: Record<string, FilePlace>) => {
      const scanId = scanIdRef.current;
      if (scanId === null) {
        return;
      }
      wait(false);
      setApplying(true);
      applyFilePlaces(scanId, filePlaces).catch((error: unknown) => {
        if (scanIdRef.current === scanId) {
          setApplying(false);
          reportError(String(error));
        }
      });
    },
    [reportError, scanIdRef, wait]
  );

  // A new destination drops the plans. They come back once the places apply again.
  const clearPlans = useCallback(() => {
    forgetPlans();
    wait(true);
    const scanId = scanIdRef.current;
    if (scanId !== null) {
      clearScanPlans(scanId).catch((error: unknown) =>
        reportError(`Failed to clear the plans: ${String(error)}`)
      );
    }
  }, [forgetPlans, reportError, scanIdRef, wait]);

  // A new scan starts with no plans and waits for its places.
  const resetPlanning = useCallback(() => {
    forgetPlans();
    wait(true);
    setApplying(false);
  }, [forgetPlans, wait]);

  return {
    movePlansByFileId,
    planErrorsByFileId,
    // Still planning: waiting for the places, or the backend is planning for them.
    planning: waiting || applying,
    applyFilePlaces: applyPlaces,
    waitingForPlaces: useCallback(() => waitingRef.current, []),
    clearPlans,
    resetPlanning
  };
}
