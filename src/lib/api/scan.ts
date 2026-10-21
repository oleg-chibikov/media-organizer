import type { ScanErrorEvent, ScanFileEvent, ScanFinishedEvent, ScanStartedEvent } from "../types";
import { listenAll } from "./listenAll";
import { invokeCommand } from "./tauriRuntime";

export const chooseFolder = () => invokeCommand<string | null>("choose_folder");
export const startScan = (selectedPath: string) =>
  invokeCommand<void>("start_scan", { selectedPath });
export const cancelScan = () => invokeCommand<void>("cancel_scan");

export interface ScanListeners {
  onStarted: (payload: ScanStartedEvent) => void;
  onFile: (payload: ScanFileEvent) => void;
  onSkipped: (payload: ScanFileEvent) => void;
  onError: (payload: ScanErrorEvent) => void;
  onFinished: (payload: ScanFinishedEvent) => void;
}

const SCAN_EVENTS = {
  onStarted: "scan:started",
  onFile: "scan:file",
  onSkipped: "scan:skipped",
  onError: "scan:error",
  onFinished: "scan:finished"
} as const;

export const attachScanListeners = (handlers: ScanListeners) => listenAll(SCAN_EVENTS, handlers);
