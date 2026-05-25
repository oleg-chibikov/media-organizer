import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  DestinationSettings,
  FileKind,
  FileRecord,
  HistoryIndexEntry,
  HistoryIndexUpdatedEvent,
  HistoryRedoProgressEvent,
  HistoryRunDetails,
  HistoryUndoProgressEvent,
  MetadataReadRequest,
  MetadataResult,
  MoveConfirmationSummary,
  MoveFileEvent,
  MoveFinishedEvent,
  MoveProgressEvent,
  MovePlan,
  PlaceLookupDiagnosticResult,
  ProcessingConfig,
  PlanErrorEvent,
  PlanFileEvent,
  PlanFinishedEvent,
  ScanErrorEvent,
  ScanFileEvent,
  ScanFinishedEvent,
  ScanStartedEvent
} from "./types";

const TAURI_RUNTIME_ERROR =
  "Tauri runtime is unavailable. Start the app with Tauri (for example `mise x -- pnpm tauri dev`), not plain `vite`/web dev mode.";

function ensureTauriRuntime(): void {
  const tauriInternals = (window as Window & { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;

  if (!tauriInternals) {
    throw new Error(TAURI_RUNTIME_ERROR);
  }
}

function invokeCommand<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  ensureTauriRuntime();
  return invoke<T>(command, args);
}

function normalizeMetadataResult(result: MetadataResult): MetadataResult {
  return {
    ...result,
    gps_coordinate: result.gps_coordinate ?? null,
    gps_source: result.gps_source ?? null,
    place_bucket_id: result.place_bucket_id ?? null,
    place_status: result.place_status ?? "not_available",
    place: result.place ?? null,
    place_error: result.place_error ?? null
  };
}

export const SCAN_EVENTS = {
  started: "scan:started",
  file: "scan:file",
  skipped: "scan:skipped",
  error: "scan:error",
  finished: "scan:finished"
} as const;

export const PLAN_EVENTS = {
  file: "plan:file",
  error: "plan:error",
  finished: "plan:finished"
} as const;

export const MOVE_EVENTS = {
  file: "move:file",
  progress: "move:progress",
  finished: "move:finished"
} as const;

export const HISTORY_EVENTS = {
  indexUpdated: "history:index-updated",
  undoProgress: "history:undo-progress",
  redoProgress: "history:redo-progress"
} as const;

export const healthCheck = () => invokeCommand<string>("health_check");
export const getProcessingConfig = () => invokeCommand<ProcessingConfig>("get_processing_config");
export const getDestinationSettings = () =>
  invokeCommand<DestinationSettings>("get_destination_settings");
export const chooseRootMediaFolder = () =>
  invokeCommand<DestinationSettings | null>("choose_root_media_folder");
export const clearRootMediaFolder = () =>
  invokeCommand<DestinationSettings>("clear_root_media_folder");
export const setForceProcessedFolder = (forceProcessedFolder: boolean) =>
  invokeCommand<DestinationSettings>("set_force_processed_folder", {
    forceProcessedFolder
  });
export const chooseFolder = () => invokeCommand<string | null>("choose_folder");
export const startScan = (selectedPath: string) =>
  invokeCommand<void>("start_scan", { selectedPath });
export const cancelScan = () => invokeCommand<void>("cancel_scan");
export const readMetadataForFile = (fileId: string, sourcePath: string, fileKind: FileKind) =>
  invokeCommand<MetadataResult>("read_metadata_for_file", {
    fileId,
    sourcePath,
    fileKind
  }).then(normalizeMetadataResult);
export const readMetadataForFiles = (requests: MetadataReadRequest[]) =>
  invokeCommand<MetadataResult[]>("read_metadata_for_files", { requests }).then((results) =>
    results.map(normalizeMetadataResult)
  );
export const testPlaceLookup = (coordinate: { latitude: number; longitude: number }) =>
  invokeCommand<PlaceLookupDiagnosticResult>("test_place_lookup", { coordinate });

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

export const executeMovePlan = (
  selectedPath: string,
  files: FileRecord[],
  metadataResults: MetadataResult[],
  plans: MovePlan[]
) =>
  invokeCommand<void>("execute_move_plan", {
    selectedPath,
    files,
    metadataResults,
    plans
  });

export const revealFileInExplorer = (sourcePath: string) =>
  invokeCommand<void>("reveal_file_in_explorer", { sourcePath });
export const deleteHistoryRun = (runId: string) =>
  invokeCommand<void>("delete_history_run", { runId });
export const listHistoryRuns = () => invokeCommand<HistoryIndexEntry[]>("list_history_runs");
export const getHistoryRun = (runId: string) =>
  invokeCommand<HistoryRunDetails | null>("get_history_run", { runId });
export const undoHistoryRun = (runId: string) => invokeCommand<void>("undo_history_run", { runId });
export const redoHistoryRun = (runId: string) => invokeCommand<void>("redo_history_run", { runId });

export interface ScanListeners {
  onStarted: (payload: ScanStartedEvent) => void;
  onFile: (payload: ScanFileEvent) => void;
  onSkipped: (payload: ScanFileEvent) => void;
  onError: (payload: ScanErrorEvent) => void;
  onFinished: (payload: ScanFinishedEvent) => void;
}

export interface PlanListeners {
  onFile: (payload: PlanFileEvent) => void;
  onError: (payload: PlanErrorEvent) => void;
  onFinished: (payload: PlanFinishedEvent) => void;
}

export interface MoveListeners {
  onProgress: (payload: MoveProgressEvent) => void;
  onFile: (payload: MoveFileEvent) => void;
  onFinished: (payload: MoveFinishedEvent) => void;
}

export interface HistoryListeners {
  onIndexUpdated: (payload: HistoryIndexUpdatedEvent) => void;
  onUndoProgress?: (payload: HistoryUndoProgressEvent) => void;
  onRedoProgress?: (payload: HistoryRedoProgressEvent) => void;
}

export async function attachScanListeners(handlers: ScanListeners): Promise<UnlistenFn[]> {
  ensureTauriRuntime();

  const listeners = await Promise.all([
    listen<ScanStartedEvent>(SCAN_EVENTS.started, (event) => handlers.onStarted(event.payload)),
    listen<ScanFileEvent>(SCAN_EVENTS.file, (event) => handlers.onFile(event.payload)),
    listen<ScanFileEvent>(SCAN_EVENTS.skipped, (event) => handlers.onSkipped(event.payload)),
    listen<ScanErrorEvent>(SCAN_EVENTS.error, (event) => handlers.onError(event.payload)),
    listen<ScanFinishedEvent>(SCAN_EVENTS.finished, (event) => handlers.onFinished(event.payload))
  ]);

  return listeners;
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

export async function attachMoveListeners(handlers: MoveListeners): Promise<UnlistenFn[]> {
  ensureTauriRuntime();

  const listeners = await Promise.all([
    listen<MoveProgressEvent>(MOVE_EVENTS.progress, (event) => handlers.onProgress(event.payload)),
    listen<MoveFileEvent>(MOVE_EVENTS.file, (event) => handlers.onFile(event.payload)),
    listen<MoveFinishedEvent>(MOVE_EVENTS.finished, (event) => handlers.onFinished(event.payload))
  ]);

  return listeners;
}

export async function attachHistoryListeners(handlers: HistoryListeners): Promise<UnlistenFn[]> {
  ensureTauriRuntime();

  const listeners = await Promise.all([
    listen<HistoryIndexUpdatedEvent>(HISTORY_EVENTS.indexUpdated, (event) =>
      handlers.onIndexUpdated(event.payload)
    )
  ]);
  if (handlers.onUndoProgress) {
    listeners.push(
      await listen<HistoryUndoProgressEvent>(HISTORY_EVENTS.undoProgress, (event) =>
        handlers.onUndoProgress?.(event.payload)
      )
    );
  }
  if (handlers.onRedoProgress) {
    listeners.push(
      await listen<HistoryRedoProgressEvent>(HISTORY_EVENTS.redoProgress, (event) =>
        handlers.onRedoProgress?.(event.payload)
      )
    );
  }

  return listeners;
}
