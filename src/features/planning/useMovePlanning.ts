import { useCallback, useRef, useState } from "react";
import { createMovePlan } from "../../lib/api";
import type { FileRecord, MetadataResult, PlaceSuggestion, PlacedMove } from "../../lib/types";
import { claimPlanCandidates, restorePlanSignatures } from "./planCandidates";
import { acceptPlaces } from "./acceptedPlaces";
import { useActiveRequests } from "./useActiveRequests";
import { usePlacedMoves } from "./usePlacedMoves";
import { usePlanBatches } from "./usePlanBatches";
import { usePlanEvents } from "./usePlanEvents";
import { withoutFileIds } from "./withoutFileIds";

interface MovePlanningOptions {
  selectedFolder: string | null;
  reportError: (message: string) => void;
  onPlacedFiles: (added: PlacedMove[], droppedIds: string[]) => void;
}

export function useMovePlanning({
  selectedFolder,
  reportError,
  onPlacedFiles
}: MovePlanningOptions) {
  const { movePlansByFileId, addPlan, flushPlans, dropPlans, dropPlansFor } = usePlanBatches();
  const { activePlanRequests, changeActiveRequests, resetActiveRequests } = useActiveRequests();
  const [planErrorsByFileId, setPlanErrorsByFileId] = useState<Record<string, string>>({});
  const signaturesRef = useRef<Map<string, string>>(new Map());
  // Accepted place suggestions go with every plan request.
  const placesRef = useRef(acceptPlaces([]));
  const { planPlaced, dropPlaced, isPlaced, resetPlaced } = usePlacedMoves({
    selectedFolder,
    addPlan,
    flushPlans,
    dropPlansFor,
    changeActiveRequests,
    reportError,
    onPlacedFiles
  });

  const requestPlans = useCallback(
    (rows: FileRecord[], metadataResults: MetadataResult[]) => {
      if (!selectedFolder) {
        return;
      }
      const places = placesRef.current;
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
      changeActiveRequests(1);
      const { files, metadata } = candidates;
      void createMovePlan(selectedFolder, files, metadata, places.suggestions).catch(
        (error: unknown) => {
          changeActiveRequests(-1);
          restorePlanSignatures(signaturesRef.current, candidates.previousSignatures);
          reportError(`Failed to create move plan: ${String(error)}`);
        }
      );
    },
    [changeActiveRequests, dropPlansFor, isPlaced, reportError, selectedFolder]
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

  // Plan events carry no request id, so a file this session didn't ask for belongs to an older scan.
  usePlanEvents({
    isRequested: (fileId) => signaturesRef.current.has(fileId),
    addPlan,
    addError: (fileId, error) =>
      setPlanErrorsByFileId((previous) => ({ ...previous, [fileId]: error })),
    flushPlans,
    changeActiveRequests,
    reportError
  });

  // A new destination or scan drops the place folders along with the plans.
  const clearPlans = useCallback(() => {
    signaturesRef.current = new Map();
    placesRef.current = acceptPlaces([]);
    dropPlaced();
    dropPlans();
    setPlanErrorsByFileId({});
  }, [dropPlaced, dropPlans]);

  const resetPlanning = useCallback(() => {
    clearPlans();
    resetPlaced();
    resetActiveRequests();
  }, [clearPlans, resetActiveRequests, resetPlaced]);

  return {
    movePlansByFileId,
    activePlanRequests,
    planErrorsByFileId,
    requestPlans,
    applyPlaceSuggestions,
    clearPlans,
    resetPlanning
  };
}
