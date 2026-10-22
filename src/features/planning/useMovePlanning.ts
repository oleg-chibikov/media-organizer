import { useCallback, useRef, useState, type RefObject } from "react";
import { createMovePlan } from "../../lib/api";
import type { FileRecord, MetadataResult, PlaceSuggestion, PlacedMove } from "../../lib/types";
import { claimPlanCandidates, restorePlanSignatures } from "./planCandidates";
import { acceptPlaces, type AcceptedPlaces } from "./acceptedPlaces";
import { usePlacedMoves } from "./usePlacedMoves";
import { usePlanBatches } from "./usePlanBatches";
import { usePlanEvents } from "./usePlanEvents";
import { usePlanRequests } from "./usePlanRequests";
import { withoutFileIds } from "./withoutFileIds";

interface MovePlanningOptions {
  selectedFolder: string | null;
  // The backend plans the files of this scan by id.
  scanIdRef: RefObject<number | null>;
  reportError: (message: string) => void;
  onPlacedFiles: (added: PlacedMove[], droppedIds: string[]) => void;
}

export function useMovePlanning({
  selectedFolder,
  scanIdRef,
  reportError,
  onPlacedFiles
}: MovePlanningOptions) {
  const { movePlansByFileId, addPlan, flushPlans, dropPlans, dropPlansFor } = usePlanBatches();
  const { activePlanRequests, startRequest, finishRequest, isActive, resetRequests } =
    usePlanRequests();
  const [planErrorsByFileId, setPlanErrorsByFileId] = useState<Record<string, string>>({});
  const signaturesRef = useRef<Map<string, string>>(new Map());
  // Accepted place suggestions go with every plan request.
  // `null` holds plans back until detection answers, so no file shows a date folder first.
  const placesRef = useRef<AcceptedPlaces | null>(acceptPlaces([]));
  const { planPlaced, dropPlaced, isPlaced, resetPlaced } = usePlacedMoves({
    selectedFolder,
    scanIdRef,
    addPlan,
    flushPlans,
    dropPlansFor,
    startRequest,
    finishRequest,
    reportError,
    onPlacedFiles
  });

  const requestPlans = useCallback(
    (rows: FileRecord[], metadataResults: MetadataResult[]) => {
      const places = placesRef.current;
      const scanId = scanIdRef.current;
      if (!selectedFolder || !places || scanId === null) {
        return;
      }
      const candidates = claimPlanCandidates(
        rows.filter((row) => !isPlaced(row.id)),
        metadataResults,
        signaturesRef.current,
        places.keys
      );
      if (candidates.files.length === 0) {
        return;
      }
      // A file planned again gets a fresh answer, so its old plan and error go.
      const fileIds = candidates.files.map((file) => file.id);
      dropPlansFor(fileIds);
      setPlanErrorsByFileId((previous) => withoutFileIds(previous, fileIds));
      const requestId = startRequest();
      void createMovePlan(requestId, selectedFolder, scanId, fileIds, places.suggestions).catch(
        (error: unknown) => {
          // After a reset the signatures belong to the new folder.
          if (!isActive(requestId)) {
            return;
          }
          finishRequest(requestId);
          restorePlanSignatures(signaturesRef.current, candidates.previousSignatures);
          reportError(`Failed to create move plan: ${String(error)}`);
        }
      );
    },
    [
      dropPlansFor,
      finishRequest,
      isActive,
      isPlaced,
      reportError,
      scanIdRef,
      selectedFolder,
      startRequest
    ]
  );

  // Plans again only the files whose place folder changed, plus the placed files that move.
  const applyPlaceSuggestions = useCallback(
    (suggestions: PlaceSuggestion[], rows: FileRecord[], metadataResults: MetadataResult[]) => {
      placesRef.current = acceptPlaces(suggestions);
      requestPlans(rows, metadataResults);
      planPlaced(suggestions, metadataResults);
    },
    [planPlaced, requestPlans]
  );

  usePlanEvents({
    isActive,
    addPlan,
    addError: (fileId, error) =>
      setPlanErrorsByFileId((previous) => ({ ...previous, [fileId]: error })),
    flushPlans,
    finishRequest,
    reportError
  });

  // A new destination or scan drops the place folders along with the plans.
  // Answers to requests in flight carry the old destination, so they are forgotten too.
  const clearPlans = useCallback(
    (waitForPlaces: boolean) => {
      signaturesRef.current = new Map();
      placesRef.current = waitForPlaces ? null : acceptPlaces([]);
      dropPlaced();
      dropPlans();
      resetRequests();
      setPlanErrorsByFileId({});
    },
    [dropPlaced, dropPlans, resetRequests]
  );

  const resetPlanning = useCallback(
    (waitForPlaces: boolean) => {
      clearPlans(waitForPlaces);
      resetPlaced();
    },
    [clearPlans, resetPlaced]
  );

  const waitingForPlaces = useCallback(() => placesRef.current === null, []);

  return {
    movePlansByFileId,
    activePlanRequests,
    planErrorsByFileId,
    requestPlans,
    applyPlaceSuggestions,
    waitingForPlaces,
    clearPlans,
    resetPlanning
  };
}
