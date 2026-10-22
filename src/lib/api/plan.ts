import type {
  FileRecord,
  MetadataResult,
  MoveConfirmationSummary,
  MovePlan,
  PlaceSuggestion,
  PlanErrorEvent,
  PlanFileEvent,
  PlanFinishedEvent
} from "../types";
import { listenAll } from "./listenAll";
import { invokeCommand } from "./tauriRuntime";

/** Files in `placeSuggestions`, the accepted ones, go to place folders instead of date folders. */
export const createMovePlan = (
  selectedPath: string,
  files: FileRecord[],
  metadataResults: MetadataResult[],
  placeSuggestions: PlaceSuggestion[]
) =>
  invokeCommand<void>("create_move_plan", {
    selectedPath,
    files,
    metadataResults,
    placeSuggestions
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
