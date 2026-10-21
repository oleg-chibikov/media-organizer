import type { MetadataResult } from "../lib/types";
import type {
  DestinationSettingsOperation,
  useDestinationSettings
} from "../features/destination-settings/useDestinationSettings";
import type { useMoveExecution } from "../features/moves/useMoveExecution";
import type { useMovePlanning } from "../features/planning/useMovePlanning";
import type { useScanSession } from "../features/scan/useScanSession";

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

  return (operation: DestinationSettingsOperation) =>
    void destination.applyChange(operation, {
      locked: moves.execution.executing,
      onChanged: replanForDestinationChange
    });
}
