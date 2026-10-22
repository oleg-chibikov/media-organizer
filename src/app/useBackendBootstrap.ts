import { useEffect, useState } from "react";
import { getDestinationSettings, healthCheck } from "../lib/api";
import type { DestinationSettings } from "../lib/types";

interface BootstrapCallbacks {
  onSettingsLoaded: (settings: DestinationSettings) => void;
  onSettingsFailed: (error: unknown) => void;
}

// Loads health and settings at startup. Each one fails on its own.
export function useBackendBootstrap({ onSettingsLoaded, onSettingsFailed }: BootstrapCallbacks) {
  const [health, setHealth] = useState<string>("checking");

  useEffect(() => {
    let ignore = false;
    const whileMounted =
      <T>(apply: (value: T) => void) =>
      (value: T) => {
        if (!ignore) {
          apply(value);
        }
      };

    healthCheck().then(
      whileMounted(setHealth),
      whileMounted((error: unknown) => setHealth(`error: ${String(error)}`))
    );
    getDestinationSettings().then(whileMounted(onSettingsLoaded), whileMounted(onSettingsFailed));
    return () => {
      ignore = true;
    };
  }, [onSettingsFailed, onSettingsLoaded]);

  return { health };
}
