import type {
  DestinationSettingsOperation,
  useDestinationSettings
} from "../features/destination-settings";
import type { useMoveExecution } from "../features/moves";
import type { useMovePlanning } from "../features/planning";
import type { useScanSession } from "../features/scan";
import { chooseRootMediaFolder, clearRootMediaFolder, setForceProcessedFolder } from "../lib/api";
import type { MetadataResult } from "../lib/types";

interface DestinationChangeOptions {
  destination: ReturnType<typeof useDestinationSettings>;
  scan: ReturnType<typeof useScanSession>;
  metadataByFileId: Record<string, MetadataResult>;
  planning: ReturnType<typeof useMovePlanning>;
  moves: ReturnType<typeof useMoveExecution>;
}

// A new destination root changes every planned path, so plans are rebuilt before execution starts.
export function useDestinationChange({
  destination,
  scan,
  metadataByFileId,
  planning,
  moves
}: DestinationChangeOptions) {
  const replanForDestinationChange = () => {
    if (!scan.selectedFolder || moves.execution.executing || moves.execution.confirmed) {
      return;
    }
    const metadataResults = scan.rows
      .map((row) => metadataByFileId[row.id])
      .filter((result): result is MetadataResult => result !== undefined);
    if (metadataResults.length === 0) {
      return;
    }
    planning.clearPlans();
    moves.resetMovesForReplan();
    planning.requestPlans(scan.rows, metadataResults);
  };

  const change = (operation: DestinationSettingsOperation) =>
    void destination.applyChange(operation, {
      locked: moves.execution.executing,
      onChanged: replanForDestinationChange
    });

  return {
    chooseRoot: () => change(chooseRootMediaFolder),
    clearRoot: () => change(clearRootMediaFolder),
    setForceProcessed: (nextValue: boolean) => change(() => setForceProcessedFolder(nextValue))
  };
}
