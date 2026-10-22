import { useEffect, useState } from "react";
import {
  attachMetadataListeners,
  attachScanListeners,
  revealFileInExplorer,
  useBackendEvents
} from "../lib/api";
import {
  groupsByPlace,
  useDestinationSettings,
  useEffectiveDestinationRoot
} from "../features/destination-settings";
import { useHistory } from "../features/history";
import { useMetadataReads } from "../features/metadata";
import { useMoveExecution } from "../features/moves";
import { useFilePlaces } from "../features/file-places";
import { useMovePlanning } from "../features/planning";
import { useScanSession } from "../features/scan";
import { useBackendBootstrap } from "./useBackendBootstrap";
import { useDestinationChange } from "./useDestinationChange";

// Wires the scan, metadata, planning and move pipeline together; each step owns its own state.
export function useAppSession() {
  const [folderError, setFolderError] = useState<string | null>(null);
  const destination = useDestinationSettings();
  const groupByPlace = groupsByPlace(destination.settings);
  const backend = useBackendBootstrap({
    onSettingsLoaded: destination.settingsLoaded,
    onSettingsFailed: destination.settingsFailed
  });
  const scan = useScanSession({
    reportError: setFolderError,
    onNewScan: () => resetPipeline()
  });
  const planning = useMovePlanning({
    scanIdRef: scan.scanIdRef,
    reportError: setFolderError
  });
  const metadata = useMetadataReads({
    scanId: scan.scanId,
    scanIdRef: scan.scanIdRef,
    reportError: setFolderError
  });
  const { completed, cancelled, discovered } = scan.progress;
  const places = useFilePlaces({
    selectedFolder: scan.selectedFolder,
    scanIdRef: scan.scanIdRef,
    rows: scan.rows,
    metadataByFileId: metadata.metadataByFileId,
    scanCompleted: completed && !cancelled,
    scanCancelled: cancelled,
    metadataBusy: metadata.reading,
    home: destination.settings?.home ?? null,
    groupByPlace,
    reportError: setFolderError,
    onApply: (filePlaces) => {
      // Files confirmed to move keep their folders. Plans still waiting take the first answer.
      const { confirmed, executing } = moves.execution;
      if ((confirmed || executing) && !planning.waitingForPlaces()) {
        return;
      }
      planning.applyFilePlaces(filePlaces);
    }
  });
  const planCompleted =
    completed &&
    !cancelled &&
    !metadata.reading &&
    !planning.planning &&
    !(groupByPlace && places.locating);
  const stillChecking = !planCompleted && !cancelled;
  const moves = useMoveExecution({
    stillChecking,
    filesFound: discovered > 0,
    selectedFolder: scan.selectedFolder,
    rows: scan.rows,
    rowByIdRef: scan.rowByIdRef,
    scanIdRef: scan.scanIdRef,
    metadataByFileId: metadata.metadataByFileId,
    movePlansByFileId: planning.movePlansByFileId,
    reportError: setFolderError
  });
  // Pausing the run also stops reading metadata, so the computer gets its CPU back.
  const { setPaused: setMetadataPaused } = metadata;
  const movesPaused = moves.execution.paused;
  useEffect(() => setMetadataPaused(movesPaused), [movesPaused, setMetadataPaused]);
  const history = useHistory(setFolderError);

  // Metadata, plans, moves and place folders belong to the old folder's files.
  const resetPipeline = () => {
    metadata.resetMetadata();
    planning.resetPlanning();
    moves.resetMoves();
    places.reset();
  };

  useBackendEvents(attachScanListeners, scan.events, (error) =>
    setFolderError(`Failed to attach scan listeners: ${String(error)}`)
  );
  useBackendEvents(attachMetadataListeners, metadata.events, (error) =>
    setFolderError(`Failed to attach metadata listeners: ${String(error)}`)
  );

  const effectiveDestination = useEffectiveDestinationRoot(
    scan.selectedFolder,
    destination.settings
  );

  const destinationActions = useDestinationChange({
    destination,
    scan,
    planning,
    moves,
    filePlaces: places
  });

  return {
    folderError,
    dismissError: () => setFolderError(null),
    openDestination: () => {
      if (effectiveDestination.rootPath) {
        revealFileInExplorer(effectiveDestination.rootPath).catch((error: unknown) =>
          setFolderError(`Failed to open the destination: ${String(error)}`)
        );
      }
    },
    health: backend.health,
    scan,
    metadataByFileId: metadata.metadataByFileId,
    groupByPlace,
    planning: {
      movePlansByFileId: planning.movePlansByFileId,
      planErrorsByFileId: planning.planErrorsByFileId,
      planErrors: Object.keys(planning.planErrorsByFileId).length,
      isPlanning: planning.planning,
      planCompleted,
      stillChecking
    },
    moves,
    places,
    history,
    destination,
    destinationActions,
    effectiveDestination
  };
}
