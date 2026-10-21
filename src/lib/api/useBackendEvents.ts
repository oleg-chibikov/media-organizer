import type { UnlistenFn } from "@tauri-apps/api/event";
import { useEffect, useRef } from "react";

type AttachListeners<THandlers> = (handlers: THandlers) => Promise<UnlistenFn[]>;

// Attaches once per `attach` and forwards each event to the latest handlers from render.
export function useBackendEvents<THandlers extends object>(
  attach: AttachListeners<THandlers>,
  handlers: THandlers,
  onAttachError: (error: unknown) => void
): void {
  const latestRef = useRef({ handlers, onAttachError });

  useEffect(() => {
    latestRef.current = { handlers, onAttachError };
  });

  useEffect(() => {
    let unsubs: UnlistenFn[] = [];
    let disposed = false;
    const forwarding = forwardToLatest(() => latestRef.current.handlers);

    void attach(forwarding)
      .then((listeners) => {
        if (disposed) {
          for (const unlisten of listeners) {
            unlisten();
          }
          return;
        }
        unsubs = listeners;
      })
      .catch((error: unknown) => {
        if (!disposed) {
          latestRef.current.onAttachError(error);
        }
      });

    return () => {
      disposed = true;
      for (const unlisten of unsubs) {
        unlisten();
      }
    };
  }, [attach]);
}

function forwardToLatest<THandlers extends object>(getHandlers: () => THandlers): THandlers {
  const keys = Object.keys(getHandlers()) as (keyof THandlers)[];
  const entries = keys.map((key) => [
    key,
    (payload: unknown) => {
      const handler = getHandlers()[key] as ((payload: unknown) => void) | undefined;
      handler?.(payload);
    }
  ]);
  return Object.fromEntries(entries) as THandlers;
}
