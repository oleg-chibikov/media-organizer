import { useState } from "react";
import { attachScanListeners, revealFileInExplorer, useBackendEvents } from "../lib/api";
import {
  useDestinationSettings,
  useEffectiveDestinationRoot
} from "../features/destination-settings";
import { useHistory } from "../features/history";
import { metadataResultsFor, useMetadataQueue } from "../features/metadata";
import { useMoveExecution } from "../features/moves";
import { usePlaceSuggestions } from "../features/place-suggestions";
import { useMovePlanning } from "../features/planning";
import { useScanSession } from "../features/scan";
import { useBackendBootstrap } from "./useBackendBootstrap";
import { useDestinationChange } from "./useDestinationChange";

// Wires the scan, metadata, planning and move pipeline together; each step owns its own state.
export function useAppSession() {
  const [folderError, setFolderError] = useState<string | null>(null);
  const destination = useDestinationSettings();
  const backend = useBackendBootstrap({
    onSettingsLoaded: destination.settingsLoaded,
    onSettingsFailed: destination.settingsFailed,
    reportError: setFolderError
  });
  const scan = useScanSession({
    reportError: setFolderError,
    onNewScan: () => resetPipeline()
  });
  const planning = useMovePlanning({
    selectedFolder: scan.selectedFolder,
    reportError: setFolderError,
    onPlacedFiles: (added, droppedIds) => {
      // Dates go first, so the metadata queue doesn't read the new rows again.
      metadata.removeResults(droppedIds);
      metadata.addResults(added.map((move) => move.metadata));
      scan.removeRows(droppedIds);
      scan.addRows(added.map((move) => move.record));
    }
  });
  const metadata = useMetadataQueue({
    rows: scan.rows,
    rowByIdRef: scan.rowByIdRef,
    concurrency: backend.metadataConcurrency,
    queueCapacity: backend.metadataQueueCapacity,
    reportError: setFolderError,
    onMetadataRead: planning.requestPlans
  });
  const { completed, cancelled, discovered } = scan.progress;
  const planCompleted =
    completed &&
    !cancelled &&
    metadata.activeMetadataRequests === 0 &&
    planning.activePlanRequests === 0;
  const stillChecking = !planCompleted && !cancelled;
  const moves = useMoveExecution({
    stillChecking,
    filesFound: discovered > 0,
    selectedFolder: scan.selectedFolder,
    rows: scan.rows,
    rowByIdRef: scan.rowByIdRef,
    metadataByFileId: metadata.metadataByFileId,
    movePlansByFileId: planning.movePlansByFileId,
    reportError: setFolderError
  });
  const places = usePlaceSuggestions({
    selectedFolder: scan.selectedFolder,
    rows: scan.rows,
    metadataByFileId: metadata.metadataByFileId,
    scanCompleted: completed && !cancelled,
    metadataBusy: metadata.activeMetadataRequests > 0,
    enabled: destination.settings?.rename_in_place === false,
    home: destination.settings?.home ?? null,
    locked: moves.execution.confirmed || moves.execution.executing,
    reportError: setFolderError,
    onApply: (accepted) =>
      planning.applyPlaceSuggestions(
        accepted,
        scan.rows,
        metadataResultsFor(scan.rows, metadata.metadataByFileId)
      )
  });
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

  const effectiveDestination = useEffectiveDestinationRoot(
    scan.selectedFolder,
    destination.settings
  );

  const destinationActions = useDestinationChange({
    destination,
    scan,
    metadataByFileId: metadata.metadataByFileId,
    planning,
    moves,
    placeSuggestions: places
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
    planning: {
      movePlansByFileId: planning.movePlansByFileId,
      planErrorsByFileId: planning.planErrorsByFileId,
      planErrors: Object.keys(planning.planErrorsByFileId).length,
      isPlanning: planning.activePlanRequests > 0,
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
