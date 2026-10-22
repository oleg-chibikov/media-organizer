import { useCallback, useRef, useState, type RefObject } from "react";
import { applyFilePlaces, clearScanPlans } from "../../lib/api";
import type { FilePlace } from "../../lib/types";
import { usePlanBatches } from "./usePlanBatches";
import { usePlanEvents } from "./usePlanEvents";

interface MovePlanningOptions {
  // The backend plans the files of this scan as their dates are read.
  scanIdRef: RefObject<number | null>;
  reportError: (message: string) => void;
}

// Shows the plans the backend makes, and tells it the place of each file.
export function useMovePlanning({ scanIdRef, reportError }: MovePlanningOptions) {
  const { movePlansByFileId, planErrorsByFileId, addPlanned, flushPlans, dropPlans } =
    usePlanBatches();
  // Plans wait for the place folders until the first apply, so no file shows a date folder first.
  const [waiting, setWaiting] = useState(true);
  const waitingRef = useRef(true);
  const [applying, setApplying] = useState(false);

  const wait = useCallback((nextWaiting: boolean) => {
    waitingRef.current = nextWaiting;
    setWaiting(nextWaiting);
  }, []);

  usePlanEvents({
    scanIdRef,
    addPlanned,
    clearPlans: dropPlans,
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
    dropPlans();
    wait(true);
    const scanId = scanIdRef.current;
    if (scanId !== null) {
      clearScanPlans(scanId).catch((error: unknown) =>
        reportError(`Failed to clear the plans: ${String(error)}`)
      );
    }
  }, [dropPlans, reportError, scanIdRef, wait]);

  // A new scan starts with no plans and waits for its places.
  const resetPlanning = useCallback(() => {
    dropPlans();
    wait(true);
    setApplying(false);
  }, [dropPlans, wait]);

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
