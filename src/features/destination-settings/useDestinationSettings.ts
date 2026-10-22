import { useCallback, useState } from "react";
import { getDestinationSettings } from "../../lib/api";
import type { DestinationSettings } from "../../lib/types";

export type DestinationSettingsOperation = () => Promise<DestinationSettings | null>;

interface ChangeContext {
  locked: boolean;
  // Gets the saved settings, so the replan doesn't wait for a render.
  onChanged: (settings: DestinationSettings) => void;
}

export function useDestinationSettings() {
  const [settings, setSettings] = useState<DestinationSettings | null>(null);
  const [loading, setLoading] = useState(true);
  const [pending, setPending] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const settingsLoaded = useCallback((loaded: DestinationSettings) => {
    setSettings(loaded);
    setError(null);
    setLoading(false);
  }, []);

  const settingsFailed = useCallback((loadError: unknown) => {
    setError(`Failed to load destination settings: ${String(loadError)}`);
    setLoading(false);
  }, []);

  // A refresh with nothing new keeps the same object, so nothing that depends on it runs again.
  const refresh = useCallback(() => {
    getDestinationSettings().then(
      (loaded) =>
        setSettings((current) => (current?.updated_at === loaded.updated_at ? current : loaded)),
      (loadError: unknown) => setError(`Failed to load destination settings: ${String(loadError)}`)
    );
  }, []);

  const applyChange = async (
    operation: DestinationSettingsOperation,
    { locked, onChanged }: ChangeContext
  ) => {
    if (locked) {
      return;
    }
    setPending(true);
    try {
      const changed = (await operation()) ?? (await getDestinationSettings());
      setSettings(changed);
      setError(null);
      onChanged(changed);
    } catch (changeError: unknown) {
      setError(`Failed to update destination settings: ${String(changeError)}`);
    } finally {
      setPending(false);
    }
  };

  return {
    settings,
    loading,
    pending,
    error,
    settingsLoaded,
    settingsFailed,
    refresh,
    applyChange
  };
}
