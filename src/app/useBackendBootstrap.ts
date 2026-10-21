import { useEffect, useState } from "react";
import { getDestinationSettings, getProcessingConfig, healthCheck } from "../lib/api";
import type { DestinationSettings } from "../lib/types";

interface BootstrapCallbacks {
  onSettingsLoaded: (settings: DestinationSettings) => void;
  onSettingsFailed: (error: unknown) => void;
  reportError: (message: string) => void;
}

// Loads health, processing limits and settings at startup. Each one fails on its own.
export function useBackendBootstrap({
  onSettingsLoaded,
  onSettingsFailed,
  reportError
}: BootstrapCallbacks) {
  const [health, setHealth] = useState<string>("checking");
  const [metadataConcurrency, setMetadataConcurrency] = useState(1);
  const [metadataQueueCapacity, setMetadataQueueCapacity] = useState(256);

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
    getProcessingConfig().then(
      whileMounted((config) => {
        setMetadataConcurrency(config.metadata_concurrency);
        setMetadataQueueCapacity(config.metadata_queue_capacity);
      }),
      whileMounted((error: unknown) =>
        reportError(`Failed to load processing limits, using defaults: ${String(error)}`)
      )
    );
    getDestinationSettings().then(whileMounted(onSettingsLoaded), whileMounted(onSettingsFailed));
    return () => {
      ignore = true;
    };
  }, [onSettingsFailed, onSettingsLoaded, reportError]);

  return { health, metadataConcurrency, metadataQueueCapacity };
}
