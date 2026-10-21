import { useCallback, useRef, useState } from "react";
import { attachPlanListeners, createMovePlan, useBackendEvents } from "../../lib/api";
import type { FileRecord, MetadataResult, MovePlan } from "../../lib/types";
import { claimPlanCandidates, restorePlanSignatures } from "./planCandidates";

interface MovePlanningOptions {
  selectedFolder: string | null;
  reportError: (message: string) => void;
}

export function useMovePlanning({ selectedFolder, reportError }: MovePlanningOptions) {
  const [movePlansByFileId, setMovePlansByFileId] = useState<Record<string, MovePlan>>({});
  const [activePlanRequests, setActivePlanRequests] = useState(0);
  const [planErrors, setPlanErrors] = useState(0);
  const signaturesRef = useRef<Map<string, string>>(new Map());
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
      const candidates = claimPlanCandidates(rows, metadataResults, signaturesRef.current);
      if (candidates.files.length === 0) {
        return;
      }
      changeActiveRequests(1);
      void createMovePlan(selectedFolder, candidates.files, candidates.metadata).catch(
        (error: unknown) => {
          changeActiveRequests(-1);
          restorePlanSignatures(signaturesRef.current, candidates.previousSignatures);
          reportError(`Failed to create move plan: ${String(error)}`);
        }
      );
    },
    [changeActiveRequests, reportError, selectedFolder]
  );

  useBackendEvents(
    attachPlanListeners,
    {
      onFile: ({ plan }) => setMovePlansByFileId((prev) => ({ ...prev, [plan.file_id]: plan })),
      onError: ({ error }) => {
        setPlanErrors((count) => count + 1);
        reportError(`Plan error: ${error}`);
      },
      onFinished: () => changeActiveRequests(-1)
    },
    (error) => reportError(`Failed to attach plan listeners: ${String(error)}`)
  );

  const clearPlans = useCallback(() => {
    signaturesRef.current = new Map();
    setMovePlansByFileId({});
  }, []);

  const resetPlanning = useCallback(() => {
    clearPlans();
    activeRequestsRef.current = 0;
    setActivePlanRequests(0);
    setPlanErrors(0);
  }, [clearPlans]);

  return {
    movePlansByFileId,
    activePlanRequests,
    planErrors,
    requestPlans,
    clearPlans,
    resetPlanning
  };
}
