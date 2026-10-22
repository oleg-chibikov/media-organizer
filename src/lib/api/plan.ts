import type {
  MoveConfirmationSummary,
  PlaceSuggestion,
  PlacedFilesEvent,
  PlanClearedEvent,
  PlanErrorEvent,
  PlanFileEvent,
  PlanSettledEvent
} from "../types";
import { listenAll } from "./listenAll";
import { invokeCommand } from "./tauriRuntime";

/** Plans every dated file of scan `scanId` for the accepted place folders, and moves the
 * placed files they continue. Files read later are planned as they come. Ends with
 * `plan:settled`. */
export const applyPlaceSuggestions = (scanId: number, placeSuggestions: PlaceSuggestion[]) =>
  invokeCommand<void>("apply_place_suggestions", { scanId, placeSuggestions });

/** Drops the scan's plans for a new destination. They return once places are applied. */
export const clearScanPlans = (scanId: number) =>
  invokeCommand<void>("clear_scan_plans", { scanId });

export const getMoveConfirmationSummary = (scanId: number, fileIds: string[]) =>
  invokeCommand<MoveConfirmationSummary>("get_move_confirmation_summary", { scanId, fileIds });

export interface PlanListeners {
  onFile: (payload: PlanFileEvent) => void;
  onError: (payload: PlanErrorEvent) => void;
  onCleared: (payload: PlanClearedEvent) => void;
  onPlaced: (payload: PlacedFilesEvent) => void;
  onSettled: (payload: PlanSettledEvent) => void;
}

const PLAN_EVENTS = {
  onFile: "plan:file",
  onError: "plan:error",
  onCleared: "plan:cleared",
  onPlaced: "plan:placed",
  onSettled: "plan:settled"
} as const;

export const attachPlanListeners = (handlers: PlanListeners) => listenAll(PLAN_EVENTS, handlers);
