import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  FileKind,
  MetadataResult,
  ScanErrorEvent,
  ScanFileEvent,
  ScanFinishedEvent,
  ScanStartedEvent
} from "./types";

const TAURI_RUNTIME_ERROR =
  "Tauri runtime is unavailable. Start the app with Tauri (for example `mise x -- pnpm tauri dev`), not plain `vite`/web dev mode.";

function ensureTauriRuntime(): void {
  const tauriInternals = (window as Window & { __TAURI_INTERNALS__?: unknown })
    .__TAURI_INTERNALS__;

  if (!tauriInternals) {
    throw new Error(TAURI_RUNTIME_ERROR);
  }
}

function invokeCommand<T>(
  command: string,
  args?: Record<string, unknown>
): Promise<T> {
  ensureTauriRuntime();
  return invoke<T>(command, args);
}

export const SCAN_EVENTS = {
  started: "scan:started",
  file: "scan:file",
  skipped: "scan:skipped",
  error: "scan:error",
  finished: "scan:finished"
} as const;

export const healthCheck = () => invokeCommand<string>("health_check");
export const chooseFolder = () => invokeCommand<string | null>("choose_folder");
export const startScan = (selectedPath: string) =>
  invokeCommand<void>("start_scan", { selectedPath });
export const cancelScan = () => invokeCommand<void>("cancel_scan");
export const readMetadataForFile = (
  fileId: string,
  sourcePath: string,
  fileKind: FileKind
) =>
  invokeCommand<MetadataResult>("read_metadata_for_file", {
    fileId,
    sourcePath,
    fileKind
  });

export interface ScanListeners {
  onStarted: (payload: ScanStartedEvent) => void;
  onFile: (payload: ScanFileEvent) => void;
  onSkipped: (payload: ScanFileEvent) => void;
  onError: (payload: ScanErrorEvent) => void;
  onFinished: (payload: ScanFinishedEvent) => void;
}

export async function attachScanListeners(
  handlers: ScanListeners
): Promise<UnlistenFn[]> {
  ensureTauriRuntime();

  const listeners = await Promise.all([
    listen<ScanStartedEvent>(SCAN_EVENTS.started, (event) =>
      handlers.onStarted(event.payload)
    ),
    listen<ScanFileEvent>(SCAN_EVENTS.file, (event) =>
      handlers.onFile(event.payload)
    ),
    listen<ScanFileEvent>(SCAN_EVENTS.skipped, (event) =>
      handlers.onSkipped(event.payload)
    ),
    listen<ScanErrorEvent>(SCAN_EVENTS.error, (event) =>
      handlers.onError(event.payload)
    ),
    listen<ScanFinishedEvent>(SCAN_EVENTS.finished, (event) =>
      handlers.onFinished(event.payload)
    )
  ]);

  return listeners;
}
