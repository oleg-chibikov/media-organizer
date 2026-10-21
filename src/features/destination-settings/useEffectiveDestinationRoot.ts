import { useEffect, useState } from "react";
import { getEffectiveDestinationRoot } from "../../lib/api";
import type { DestinationSettings } from "../../lib/types";

export interface EffectiveDestination {
  rootPath: string | null;
  error: string | null;
}

interface Resolved extends EffectiveDestination {
  folder: string;
  settings: DestinationSettings;
}

const UNKNOWN: EffectiveDestination = { rootPath: null, error: null };

// Asks the backend where files from the selected folder go under the current settings.
export function useEffectiveDestinationRoot(
  selectedFolder: string | null,
  settings: DestinationSettings | null
): EffectiveDestination {
  const [resolved, setResolved] = useState<Resolved | null>(null);

  useEffect(() => {
    if (!selectedFolder || !settings) {
      return;
    }
    let ignore = false;
    const keep = (result: EffectiveDestination) => {
      if (!ignore) {
        setResolved({ ...result, folder: selectedFolder, settings });
      }
    };
    getEffectiveDestinationRoot(selectedFolder).then(
      (root) => keep({ rootPath: root.root_path, error: null }),
      (error: unknown) => keep({ rootPath: null, error: String(error) })
    );
    return () => {
      ignore = true;
    };
  }, [selectedFolder, settings]);

  // An answer for an older folder or older settings is out of date.
  const isCurrent = resolved?.folder === selectedFolder && resolved.settings === settings;
  return isCurrent ? { rootPath: resolved.rootPath, error: resolved.error } : UNKNOWN;
}
