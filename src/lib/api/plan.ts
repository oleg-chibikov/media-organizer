import type {
  FilePlace,
  MoveConfirmationSummary,
  PlanClearedEvent,
  PlanFilesEvent,
  PlanSettledEvent
} from "../types";
import { listenAll } from "./listenAll";
import { invokeCommand } from "./tauriRuntime";

/** Plans every dated file of scan `scanId` into its place folder, or a date folder when
 * `filePlaces` has none for it. Files read later are planned as they come. Ends with
 * `plan:settled`. Plans of a `preview` wait for the final places before any file moves. */
export const applyFilePlaces = (
  scanId: number,
  filePlaces: Record<string, FilePlace>,
  preview: boolean
) => invokeCommand<void>("apply_file_places", { scanId, filePlaces, preview });

/** Drops the scan's plans for a new destination. They return once places are applied. */
export const clearScanPlans = (scanId: number) =>
  invokeCommand<void>("clear_scan_plans", { scanId });

export const getMoveConfirmationSummary = (scanId: number, fileIds: string[]) =>
  invokeCommand<MoveConfirmationSummary>("get_move_confirmation_summary", { scanId, fileIds });

export interface PlanListeners {
  onFiles: (payload: PlanFilesEvent) => void;
  onCleared: (payload: PlanClearedEvent) => void;
  onSettled: (payload: PlanSettledEvent) => void;
}

const PLAN_EVENTS = {
  onFiles: "plan:files",
  onCleared: "plan:cleared",
  onSettled: "plan:settled"
} as const;

export const attachPlanListeners = (handlers: PlanListeners) => listenAll(PLAN_EVENTS, handlers);
