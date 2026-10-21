import { useEffect, useState } from "react";
import { getDestinationSettings, getProcessingConfig, healthCheck } from "../lib/api";
import type { DestinationSettings } from "../lib/types";

interface BootstrapCallbacks {
  onSettingsLoaded: (settings: DestinationSettings) => void;
  onSettingsFailed: (error: unknown) => void;
}

// Loads health, processing config and destination settings together, as one startup step.
export function useBackendBootstrap({ onSettingsLoaded, onSettingsFailed }: BootstrapCallbacks) {
  const [health, setHealth] = useState<string>("checking");
  const [metadataConcurrency, setMetadataConcurrency] = useState(1);
  const [metadataQueueCapacity, setMetadataQueueCapacity] = useState(256);

  useEffect(() => {
    let ignore = false;
    Promise.all([healthCheck(), getProcessingConfig(), getDestinationSettings()])
      .then(([value, config, settings]) => {
        if (ignore) {
          return;
        }
        setHealth(value);
        setMetadataConcurrency(config.metadata_concurrency);
        setMetadataQueueCapacity(config.metadata_queue_capacity);
        onSettingsLoaded(settings);
      })
      .catch((error: unknown) => {
        if (!ignore) {
          setHealth(`error: ${String(error)}`);
          onSettingsFailed(error);
        }
      });
    return () => {
      ignore = true;
    };
  }, [onSettingsFailed, onSettingsLoaded]);

  return { health, metadataConcurrency, metadataQueueCapacity };
}
