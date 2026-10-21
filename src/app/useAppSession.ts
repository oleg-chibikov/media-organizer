import { useState } from "react";
import { attachScanListeners, useBackendEvents } from "../lib/api";
import { useDestinationSettings } from "../features/destination-settings";
import { useHistory } from "../features/history";
import { useMetadataQueue } from "../features/metadata";
import { useMoveExecution } from "../features/moves";
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
    onSettingsFailed: destination.settingsFailed
  });
  const scan = useScanSession(setFolderError);
  const planning = useMovePlanning({
    selectedFolder: scan.selectedFolder,
    reportError: setFolderError
  });
  const metadata = useMetadataQueue({
    rows: scan.rows,
    rowByIdRef: scan.rowByIdRef,
    concurrency: backend.metadataConcurrency,
    queueCapacity: backend.metadataQueueCapacity,
    reportError: setFolderError,
    onMetadataRead: planning.requestPlans
  });
  const moves = useMoveExecution({
    selectedFolder: scan.selectedFolder,
    rows: scan.rows,
    rowByIdRef: scan.rowByIdRef,
    metadataByFileId: metadata.metadataByFileId,
    movePlansByFileId: planning.movePlansByFileId,
    reportError: setFolderError
  });
  const history = useHistory(setFolderError);

  useBackendEvents(
    attachScanListeners,
    {
      ...scan.events,
      onStarted: (payload) => {
        scan.events.onStarted(payload);
        metadata.resetMetadata();
        planning.resetPlanning();
        moves.resetMoves();
      }
    },
    (error) => setFolderError(`Failed to attach scan listeners: ${String(error)}`)
  );

  const destinationActions = useDestinationChange({
    destination,
    scan,
    metadataByFileId: metadata.metadataByFileId,
    planning,
    moves
  });

  const { activeMetadataRequests } = metadata;
  const { activePlanRequests } = planning;
  const { completed, cancelled } = scan.progress;

  return {
    folderError,
    health: backend.health,
    scan,
    metadataByFileId: metadata.metadataByFileId,
    planning: {
      movePlansByFileId: planning.movePlansByFileId,
      planErrors: planning.planErrors,
      isPlanning: activePlanRequests > 0,
      planCompleted:
        completed && !cancelled && activeMetadataRequests === 0 && activePlanRequests === 0
    },
    moves,
    history,
    destination,
    destinationActions
  };
}
