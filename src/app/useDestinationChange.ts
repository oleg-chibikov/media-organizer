import { useEffect, useRef } from "react";
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
export function useDestinationChange(options: DestinationChangeOptions) {
  const { destination, moves } = options;
  // Files keep moving after confirmation as they become ready, so the root they go to stays fixed.
  const locked = moves.execution.confirmed || moves.execution.executing;
  // The change awaits a picker; the replan must see the session as it is when the change lands.
  const latestRef = useRef(options);
  useEffect(() => {
    latestRef.current = options;
  });

  const replanForDestinationChange = () => {
    const { scan, metadataByFileId, planning, moves: latestMoves } = latestRef.current;
    const { executing, confirmed } = latestMoves.execution;
    if (!scan.selectedFolder || executing || confirmed) {
      return;
    }
    const metadataResults = scan.rows
      .map((row) => metadataByFileId[row.id])
      .filter((result): result is MetadataResult => result !== undefined);
    if (metadataResults.length === 0) {
      return;
    }
    planning.clearPlans();
    latestMoves.resetMovesForReplan();
    planning.requestPlans(scan.rows, metadataResults);
  };

  const change = (operation: DestinationSettingsOperation) =>
    void destination.applyChange(operation, { locked, onChanged: replanForDestinationChange });

  return {
    locked,
    chooseRoot: () => change(chooseRootMediaFolder),
    clearRoot: () => change(clearRootMediaFolder),
    setForceProcessed: (nextValue: boolean) => change(() => setForceProcessedFolder(nextValue))
  };
}
