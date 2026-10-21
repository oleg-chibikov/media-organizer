import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { ensureTauriRuntime } from "./tauriRuntime";

type EventHandler = (payload: never) => void;

// Subscribes each provided handler to its backend event name and returns the unlisten functions.
export async function listenAll<THandlers extends { [Key in keyof THandlers]?: EventHandler }>(
  eventNames: { [Key in keyof THandlers]-?: string },
  handlers: THandlers
): Promise<UnlistenFn[]> {
  ensureTauriRuntime();
  const providedKeys = (Object.keys(handlers) as (keyof THandlers)[]).filter(
    (key) => handlers[key] !== undefined
  );
  return Promise.all(
    providedKeys.map((key) =>
      listen(eventNames[key], (event) => {
        (handlers[key] as (payload: unknown) => void)(event.payload);
      })
    )
  );
}
