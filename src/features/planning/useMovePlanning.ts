import { useCallback, useRef, useState } from "react";
import { attachPlanListeners, createMovePlan, useBackendEvents } from "../../lib/api";
import type { FileRecord, MetadataResult, PlaceSuggestion } from "../../lib/types";
import { claimPlanCandidates, restorePlanSignatures } from "./planCandidates";
import { acceptPlaces } from "./acceptedPlaces";
import { usePlanBatches } from "./usePlanBatches";
import { withoutFileIds } from "./withoutFileIds";

interface MovePlanningOptions {
  selectedFolder: string | null;
  reportError: (message: string) => void;
}

export function useMovePlanning({ selectedFolder, reportError }: MovePlanningOptions) {
  const { movePlansByFileId, addPlan, flushPlans, dropPlans, dropPlansFor } = usePlanBatches();
  const [activePlanRequests, setActivePlanRequests] = useState(0);
  const [planErrorsByFileId, setPlanErrorsByFileId] = useState<Record<string, string>>({});
  const signaturesRef = useRef<Map<string, string>>(new Map());
  // Accepted place suggestions go with every plan request.
  const placesRef = useRef(acceptPlaces([]));
  const activeRequestsRef = useRef(0);

  const changeActiveRequests = useCallback((delta: number) => {
    activeRequestsRef.current = Math.max(0, activeRequestsRef.current + delta);
    setActivePlanRequests(activeRequestsRef.current);
  }, []);

  const requestPlans = useCallback(
    (rows: FileRecord[], metadataResults: MetadataResult[]) => {
      if (!selectedFolder) {
        return;
      }
      const places = placesRef.current;
      const candidates = claimPlanCandidates(
        rows,
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
    [changeActiveRequests, dropPlansFor, reportError, selectedFolder]
  );

  // Plans again only the files whose place folder changed.
  const applyPlaceSuggestions = useCallback(
    (suggestions: PlaceSuggestion[], rows: FileRecord[], metadataResults: MetadataResult[]) => {
      placesRef.current = acceptPlaces(suggestions);
      requestPlans(rows, metadataResults);
    },
    [requestPlans]
  );

  // Plan events carry no request id, so a file this session didn't ask for belongs to an older scan.
  const isRequested = (fileId: string) => signaturesRef.current.has(fileId);

  useBackendEvents(
    attachPlanListeners,
    {
      onFile: ({ plan }) => {
        if (isRequested(plan.file_id)) {
          addPlan(plan);
        }
      },
      onError: ({ file_id, error }) => {
        if (isRequested(file_id)) {
          setPlanErrorsByFileId((previous) => ({ ...previous, [file_id]: error }));
        }
      },
      onFinished: () => {
        // Plans land before the request count drops, so planning never looks done too early.
        flushPlans();
        changeActiveRequests(-1);
      }
    },
    (error) => reportError(`Failed to attach plan listeners: ${String(error)}`)
  );

  // A new destination or scan drops the place folders along with the plans.
  const clearPlans = useCallback(() => {
    signaturesRef.current = new Map();
    placesRef.current = acceptPlaces([]);
    dropPlans();
    setPlanErrorsByFileId({});
  }, [dropPlans]);

  const resetPlanning = useCallback(() => {
    clearPlans();
    activeRequestsRef.current = 0;
    setActivePlanRequests(0);
  }, [clearPlans]);

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
