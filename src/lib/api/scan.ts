import type { ScanErrorEvent, ScanFilesEvent, ScanFinishedEvent, ScanStartedEvent } from "../types";
import { listenAll } from "./listenAll";
import { invokeCommand } from "./tauriRuntime";

export const chooseFolder = () => invokeCommand<string | null>("choose_folder");
export const startScan = (selectedPath: string) =>
  invokeCommand<void>("start_scan", { selectedPath });
export const cancelScan = () => invokeCommand<void>("cancel_scan");

export interface ScanListeners {
  onStarted: (payload: ScanStartedEvent) => void;
  onFiles: (payload: ScanFilesEvent) => void;
  onError: (payload: ScanErrorEvent) => void;
  onFinished: (payload: ScanFinishedEvent) => void;
}

const SCAN_EVENTS = {
  onStarted: "scan:started",
  onFiles: "scan:files",
  onError: "scan:error",
  onFinished: "scan:finished"
} as const;

export const attachScanListeners = (handlers: ScanListeners) => listenAll(SCAN_EVENTS, handlers);
