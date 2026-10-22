import type {
  MoveConfirmationSummary,
  MovePlan,
  PlaceSuggestion,
  PlacedMove,
  PlanErrorEvent,
  PlanFileEvent,
  PlanFinishedEvent
} from "../types";
import { listenAll } from "./listenAll";
import { invokeCommand } from "./tauriRuntime";

/** Plans the files of scan `scanId` by id. The backend kept their records and dates.
 * Files in `placeSuggestions`, the accepted ones, go to place folders instead of date folders.
 * Every `plan:*` event of the request carries `requestId`. */
export const createMovePlan = (
  requestId: number,
  selectedPath: string,
  scanId: number,
  fileIds: string[],
  placeSuggestions: PlaceSuggestion[]
) =>
  invokeCommand<void>("create_move_plan", {
    requestId,
    selectedPath,
    scanId,
    fileIds,
    placeSuggestions
  });

/** Files earlier runs placed in the folders the suggestions continue, planned into the
 * new day folders those folders get. Files that stay where they are are left out. */
export const planContinuedMoves = (
  selectedPath: string,
  scanId: number,
  fileIds: string[],
  placeSuggestions: PlaceSuggestion[]
) =>
  invokeCommand<PlacedMove[]>("plan_continued_moves", {
    selectedPath,
    scanId,
    fileIds,
    placeSuggestions
  });

export const getMoveConfirmationSummary = (scanId: number, plans: MovePlan[]) =>
  invokeCommand<MoveConfirmationSummary>("get_move_confirmation_summary", { scanId, plans });

export interface PlanListeners {
  onFile: (payload: PlanFileEvent) => void;
  onError: (payload: PlanErrorEvent) => void;
  onFinished: (payload: PlanFinishedEvent) => void;
}

const PLAN_EVENTS = {
  onFile: "plan:file",
  onError: "plan:error",
  onFinished: "plan:finished"
} as const;

export const attachPlanListeners = (handlers: PlanListeners) => listenAll(PLAN_EVENTS, handlers);
