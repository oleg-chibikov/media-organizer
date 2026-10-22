import { useEffect, useRef } from "react";
import {
  type DestinationSettingsOperation,
  type useDestinationSettings
} from "../features/destination-settings";
import type { useMoveExecution } from "../features/moves";
import type { useMovePlanning } from "../features/planning";
import type { useScanSession } from "../features/scan";
import {
  chooseRootMediaFolder,
  clearRootMediaFolder,
  setForceProcessedFolder,
  setGroupByPlace,
  setHomeFromCurrentLocation,
  setHomeLocation,
  setRenameInPlace
} from "../lib/api";
import type { HomeLocation } from "../lib/types";

interface DestinationChangeOptions {
  destination: ReturnType<typeof useDestinationSettings>;
  scan: ReturnType<typeof useScanSession>;
  planning: ReturnType<typeof useMovePlanning>;
  moves: ReturnType<typeof useMoveExecution>;
  filePlaces: { reset: () => void };
}

// A new root or mode changes every planned path, so plans are rebuilt before execution starts.
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
    const { scan, planning, moves: latestMoves } = latestRef.current;
    const { executing, confirmed } = latestMoves.execution;
    if (!scan.selectedFolder || executing || confirmed) {
      return;
    }
    // Place folders were suggested for the old destination. Detection applies them again,
    // and the backend plans every dated file for the new destination then.
    latestRef.current.filePlaces.reset();
    planning.clearPlans();
    latestMoves.resetMoves();
  };

  const change = (operation: DestinationSettingsOperation) =>
    void destination.applyChange(operation, { locked, onChanged: replanForDestinationChange });

  // Home moves no file, so it stays editable during moves. Place detection reruns on its own.
  const changeHome = (operation: DestinationSettingsOperation) =>
    destination.applyChange(operation, { locked: false, onChanged: () => undefined });

  return {
    locked,
    chooseRoot: () => change(chooseRootMediaFolder),
    clearRoot: () => change(clearRootMediaFolder),
    setForceProcessed: (nextValue: boolean) => change(() => setForceProcessedFolder(nextValue)),
    setRenameInPlace: (nextValue: boolean) => change(() => setRenameInPlace(nextValue)),
    setGroupByPlace: (nextValue: boolean) => change(() => setGroupByPlace(nextValue)),
    setHome: (home: HomeLocation | null) => void changeHome(() => setHomeLocation(home)),
    locateHome: () => changeHome(setHomeFromCurrentLocation)
  };
}
