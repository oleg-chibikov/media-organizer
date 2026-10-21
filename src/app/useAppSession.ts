import { useState } from "react";
import { attachScanListeners, useBackendEvents } from "../lib/api";
import type { MetadataResult } from "../lib/types";
import {
  useDestinationSettings,
  type DestinationSettingsOperation
} from "../features/destination-settings/useDestinationSettings";
import { useHistory } from "../features/history/useHistory";
import { useMetadataQueue } from "../features/metadata/useMetadataQueue";
import { useMoveExecution } from "../features/moves/useMoveExecution";
import { useMovePlanning } from "../features/planning/useMovePlanning";
import { useScanSession } from "../features/scan/useScanSession";
import { useBackendBootstrap } from "./useBackendBootstrap";

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

  const replanForDestinationChange = () => {
    if (!scan.selectedFolder || moves.execution.executing || moves.execution.confirmed) {
      return;
    }
    const metadataResults = scan.rows
      .map((row) => metadata.metadataByFileId[row.id])
      .filter((result): result is MetadataResult => result !== undefined);
    if (metadataResults.length === 0) {
      return;
    }
    planning.clearPlans();
    moves.resetMovesForReplan();
    planning.requestPlans(scan.rows, metadataResults);
  };

  const changeDestinationSettings = (operation: DestinationSettingsOperation) =>
    void destination.applyChange(operation, {
      locked: moves.execution.executing,
      onChanged: replanForDestinationChange
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
    changeDestinationSettings
  };
}
