import { useCallback, useRef, useState } from "react";
import { attachPlanListeners, createMovePlan, useBackendEvents } from "../../lib/api";
import type { FileRecord, MetadataResult } from "../../lib/types";
import { claimPlanCandidates, restorePlanSignatures } from "./planCandidates";
import { usePlanBatches } from "./usePlanBatches";

function withoutFiles(errors: Record<string, string>, files: FileRecord[]) {
  if (!files.some((file) => file.id in errors)) {
    return errors;
  }
  const next = { ...errors };
  for (const file of files) {
    delete next[file.id];
  }
  return next;
}

interface MovePlanningOptions {
  selectedFolder: string | null;
  reportError: (message: string) => void;
}

export function useMovePlanning({ selectedFolder, reportError }: MovePlanningOptions) {
  const { movePlansByFileId, addPlan, flushPlans, dropPlans } = usePlanBatches();
  const [activePlanRequests, setActivePlanRequests] = useState(0);
  const [planErrorsByFileId, setPlanErrorsByFileId] = useState<Record<string, string>>({});
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
      // A file planned again gets a fresh answer, so its old error goes.
      setPlanErrorsByFileId((previous) => withoutFiles(previous, candidates.files));
      changeActiveRequests(1);
      void createMovePlan(selectedFolder, candidates.files, candidates.metadata, []).catch(
        (error: unknown) => {
          changeActiveRequests(-1);
          restorePlanSignatures(signaturesRef.current, candidates.previousSignatures);
          reportError(`Failed to create move plan: ${String(error)}`);
        }
      );
    },
    [changeActiveRequests, reportError, selectedFolder]
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

  const clearPlans = useCallback(() => {
    signaturesRef.current = new Map();
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
    clearPlans,
    resetPlanning
  };
}
