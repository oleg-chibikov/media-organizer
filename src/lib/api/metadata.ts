import type { MetadataFinishedEvent, MetadataResultsEvent } from "../types";
import { listenAll } from "./listenAll";
import { invokeCommand } from "./tauriRuntime";

export const setMetadataPaused = (paused: boolean) =>
  invokeCommand<void>("set_metadata_paused", { paused });

export interface MetadataListeners {
  onResults: (payload: MetadataResultsEvent) => void;
  onFinished: (payload: MetadataFinishedEvent) => void;
}

const METADATA_EVENTS = {
  onResults: "metadata:results",
  onFinished: "metadata:finished"
} as const;

export const attachMetadataListeners = (handlers: MetadataListeners) =>
  listenAll(METADATA_EVENTS, handlers);
