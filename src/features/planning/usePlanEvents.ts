import type { RefObject } from "react";
import { attachPlanListeners, useBackendEvents } from "../../lib/api";
import type {
  MovePlan,
  PlacedFilesEvent,
  PlacedMove,
  PlanErrorEvent,
  PlanFileEvent
} from "../../lib/types";

interface PlanEventsOptions {
  scanIdRef: RefObject<number | null>;
  addPlan: (plan: MovePlan) => void;
  setError: (fileId: string, error: string) => void;
  clearPlans: () => void;
  onPlacedFiles: (added: PlacedMove[], droppedIds: string[]) => void;
  onSettled: () => void;
  reportError: (message: string) => void;
}

// Mirrors the backend's plans for the scan on screen. The backend sends them in order,
// so applying each event as it lands keeps the plans it has. Older scans' events are dropped.
export function usePlanEvents(options: PlanEventsOptions) {
  const { scanIdRef, addPlan, setError, clearPlans, onPlacedFiles, onSettled } = options;
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
      onFile: forScan(({ plan }: PlanFileEvent) => addPlan(plan)),
      onError: forScan(({ file_id, error }: PlanErrorEvent) => setError(file_id, error)),
      onCleared: forScan(clearPlans),
      onPlaced: forScan(({ added, dropped }: PlacedFilesEvent) => onPlacedFiles(added, dropped)),
      onSettled: forScan(onSettled)
    },
    (error) => options.reportError(`Failed to attach plan listeners: ${String(error)}`)
  );
}
