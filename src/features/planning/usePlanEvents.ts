import type { RefObject } from "react";
import { attachPlanListeners, useBackendEvents } from "../../lib/api";
import type { PlanFilesEvent } from "../../lib/types";
import type { PlannedFiles } from "./usePlanBatches";

interface PlanEventsOptions {
  scanIdRef: RefObject<number | null>;
  addPlanned: (planned: PlannedFiles) => void;
  clearPlans: () => void;
  onSettled: () => void;
  reportError: (message: string) => void;
}

// Mirrors the backend's plans for the scan on screen. The backend sends them in order,
// so applying each event as it lands keeps the plans it has. Older scans' events are dropped.
export function usePlanEvents(options: PlanEventsOptions) {
  const { scanIdRef, addPlanned, clearPlans, onSettled } = options;
  const forScan =
    <T extends { scan_id: number }>(apply: (payload: T) => void) =>
    (payload: T) => {
      if (payload.scan_id === scanIdRef.current) {
        apply(payload);
      }
    };
  useBackendEvents(
    attachPlanListeners,
    {
      onFiles: forScan((planned: PlanFilesEvent) => addPlanned(planned)),
      onCleared: forScan(clearPlans),
      onSettled: forScan(onSettled)
    },
    (error) => options.reportError(`Failed to attach plan listeners: ${String(error)}`)
  );
}
