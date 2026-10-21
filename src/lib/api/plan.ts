import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  FileRecord,
  MetadataResult,
  MoveConfirmationSummary,
  MovePlan,
  PlanErrorEvent,
  PlanFileEvent,
  PlanFinishedEvent
} from "../types";
import { ensureTauriRuntime, invokeCommand } from "./tauriRuntime";

export const PLAN_EVENTS = {
  file: "plan:file",
  error: "plan:error",
  finished: "plan:finished"
} as const;

export const createMovePlan = (
  selectedPath: string,
  files: FileRecord[],
  metadataResults: MetadataResult[]
) =>
  invokeCommand<void>("create_move_plan", {
    selectedPath,
    files,
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

export async function attachPlanListeners(handlers: PlanListeners): Promise<UnlistenFn[]> {
  ensureTauriRuntime();

  const listeners = await Promise.all([
    listen<PlanFileEvent>(PLAN_EVENTS.file, (event) => handlers.onFile(event.payload)),
    listen<PlanErrorEvent>(PLAN_EVENTS.error, (event) => handlers.onError(event.payload)),
    listen<PlanFinishedEvent>(PLAN_EVENTS.finished, (event) => handlers.onFinished(event.payload))
  ]);

  return listeners;
}
