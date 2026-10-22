import type {
  FileRecord,
  MetadataResult,
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

/** Files in `placeSuggestions`, the accepted ones, go to place folders instead of date folders.
 * Every `plan:*` event of the request carries `requestId`. */
export const createMovePlan = (
  requestId: number,
  selectedPath: string,
  files: FileRecord[],
  metadataResults: MetadataResult[],
  placeSuggestions: PlaceSuggestion[]
) =>
  invokeCommand<void>("create_move_plan", {
    requestId,
    selectedPath,
    files,
    metadataResults,
    placeSuggestions
  });

/** Files earlier runs placed in the folders the suggestions continue, planned into the
 * new day folders those folders get. Files that stay where they are are left out. */
export const planContinuedMoves = (
  selectedPath: string,
  placeSuggestions: PlaceSuggestion[],
  metadataResults: MetadataResult[]
) =>
  invokeCommand<PlacedMove[]>("plan_continued_moves", {
    selectedPath,
    placeSuggestions,
    metadataResults
  });

export const getMoveConfirmationSummary = (
  files: FileRecord[],
  metadataResults: MetadataResult[],
  plans: MovePlan[]
) =>
  invokeCommand<MoveConfirmationSummary>("get_move_confirmation_summary", {
    files,
    metadataResults,
    plans
  });

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
