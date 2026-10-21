import { useCallback, useState } from "react";
import { getDestinationSettings } from "../../lib/api";
import type { DestinationSettings } from "../../lib/types";

export type DestinationSettingsOperation = () => Promise<DestinationSettings | null>;

interface ChangeContext {
  locked: boolean;
  onChanged: () => void;
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

  const applyChange = async (
    operation: DestinationSettingsOperation,
    { locked, onChanged }: ChangeContext
  ) => {
    if (locked) {
      return;
    }
    setPending(true);
    try {
      setSettings((await operation()) ?? (await getDestinationSettings()));
      setError(null);
      onChanged();
    } catch (changeError: unknown) {
      setError(`Failed to update destination settings: ${String(changeError)}`);
    } finally {
      setPending(false);
    }
  };

  return { settings, loading, pending, error, settingsLoaded, settingsFailed, applyChange };
}
