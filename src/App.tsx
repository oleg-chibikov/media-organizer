import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { FileTable } from "./components/FileTable";
import { StatusBar } from "./components/StatusBar";
import { Toolbar } from "./components/Toolbar";
import { DestinationSettingsMenu } from "./components/DestinationSettingsMenu";
import { Button } from "./components/Button";
import {
  attachHistoryListeners,
  attachMoveListeners,
  attachPlanListeners,
  attachScanListeners,
  cancelScan,
  chooseRootMediaFolder,
  chooseFolder,
  clearRootMediaFolder,
  createMovePlan,
  deleteHistoryRun,
  executeMovePlan,
  getDestinationSettings,
  getHistoryRun,
  getProcessingConfig,
  healthCheck,
  listHistoryRuns,
  redoHistoryRun,
  readMetadataForFiles,
  setForceProcessedFolder,
  startScan,
  undoHistoryRun
} from "./lib/api";
import type {
  DestinationSettings,
  FileRecord,
  HistoryIndexEntry,
  HistoryRedoProgressEvent,
  HistoryRunDetails,
  HistoryUndoProgressEvent,
  MetadataResult,
  MoveConfirmationSummary,
  MovePlan,
  MoveResult,
  ScanFinishedEvent
} from "./lib/types";

const MOVE_DISPATCH_CONCURRENCY = 1;
const METADATA_BATCH_SIZE = 8;
type HistoryActionKind = "undo" | "redo";

function isProcessedPath(folder: string): boolean {
  return folder
    .toLowerCase()
    .split(/[/\\]+/)
    .some((segment) => segment === "processed");
}

function metadataPlanningSignature(metadata: MetadataResult): string {
  return `${metadata.metadata_status}:${metadata.chosen_date ?? ""}`;
}

function isMoveResultTerminal(result: MoveResult | undefined): boolean {
  return result !== undefined;
}

export default function App() {
  const [health, setHealth] = useState<string>("checking");
  const [selectedFolder, setSelectedFolder] = useState<string | null>(null);
  const [folderError, setFolderError] = useState<string | null>(null);
  const [isScanning, setIsScanning] = useState(false);
  const [discovered, setDiscovered] = useState(0);
  const [skipped, setSkipped] = useState(0);
  const [errors, setErrors] = useState(0);
  const [completed, setCompleted] = useState(false);
  const [cancelled, setCancelled] = useState(false);
  const [rows, setRows] = useState<FileRecord[]>([]);
  const [metadataByFileId, setMetadataByFileId] = useState<Record<string, MetadataResult>>({});
  const [movePlansByFileId, setMovePlansByFileId] = useState<Record<string, MovePlan>>({});
  const [moveResultsByFileId, setMoveResultsByFileId] = useState<Record<string, MoveResult>>({});
  const [metadataConcurrency, setMetadataConcurrency] = useState(1);
  const [metadataQueueCapacity, setMetadataQueueCapacity] = useState(256);
  const [activeMetadataRequests, setActiveMetadataRequests] = useState(0);
  const [activePlanRequests, setActivePlanRequests] = useState(0);
  const [planErrors, setPlanErrors] = useState(0);
  const [executionConfirmed, setExecutionConfirmed] = useState(false);
  const [isMoveDispatchPaused, setIsMoveDispatchPaused] = useState(false);
  const [isExecutingMoves, setIsExecutingMoves] = useState(false);
  const [moveCompleted, setMoveCompleted] = useState(false);
  const [movedCount, setMovedCount] = useState(0);
  const [copiedCount, setCopiedCount] = useState(0);
  const [moveSkippedCount, setMoveSkippedCount] = useState(0);
  const [moveFailedCount, setMoveFailedCount] = useState(0);
  const [historyRuns, setHistoryRuns] = useState<HistoryIndexEntry[]>([]);
  const [selectedHistoryRunId, setSelectedHistoryRunId] = useState<string | null>(null);
  const [selectedHistoryRunDetails, setSelectedHistoryRunDetails] =
    useState<HistoryRunDetails | null>(null);
  const [historyLoading, setHistoryLoading] = useState(false);
  const [historyDetailsLoading, setHistoryDetailsLoading] = useState(false);
  const [historyError, setHistoryError] = useState<string | null>(null);
  const [destinationSettings, setDestinationSettings] = useState<DestinationSettings | null>(null);
  const [destinationSettingsLoading, setDestinationSettingsLoading] = useState(true);
  const [destinationSettingsPending, setDestinationSettingsPending] = useState(false);
  const [destinationSettingsError, setDestinationSettingsError] = useState<string | null>(null);
  const isPlanning = activePlanRequests > 0;
  const planCompleted =
    completed && !cancelled && activeMetadataRequests === 0 && activePlanRequests === 0;
  const destinationSettingsLocked = isExecutingMoves;

  const seenRowIdsRef = useRef<Set<string>>(new Set());
  const requestedMetadataIdsRef = useRef<Set<string>>(new Set());
  const queuedMetadataIdsRef = useRef<Set<string>>(new Set());
  const metadataQueueRef = useRef<string[]>([]);
  const activeMetadataRequestsRef = useRef(0);
  const rowByIdRef = useRef<Map<string, FileRecord>>(new Map());
  const pumpMetadataQueueRef = useRef<() => void>(() => undefined);
  const plannedMetadataSignatureByFileIdRef = useRef<Map<string, string>>(new Map());
  const activePlanRequestsRef = useRef(0);
  const queuedMoveIdsRef = useRef<Set<string>>(new Set());
  const moveQueueRef = useRef<string[]>([]);
  const inflightMoveIdsRef = useRef<Set<string>>(new Set());
  const completedMoveIdsRef = useRef<Set<string>>(new Set());
  const trackedMoveIdsRef = useRef<Set<string>>(new Set());
  const activeMoveRequestsRef = useRef(0);
  const pumpMoveQueueRef = useRef<() => void>(() => undefined);
  const activeHistoryActionRef = useRef<{
    runId: string;
    action: HistoryActionKind;
    blockedOrFailed: number;
  } | null>(null);
  const pendingExecutableCount = Object.values(movePlansByFileId).filter((plan) => {
    if (plan.plan_status !== "ready") {
      return false;
    }
    return !isMoveResultTerminal(moveResultsByFileId[plan.file_id]);
  }).length;
  const hasPendingExecutableItems = pendingExecutableCount > 0;
  const confirmationSummary = useMemo<MoveConfirmationSummary>(() => {
    const plans = Object.values(movePlansByFileId);
    const rowsById = new Map(rows.map((row) => [row.id, row]));

    let readyOperations = 0;
    let blockedNoDate = 0;
    let blockedConflict = 0;
    let planErrorsCount = 0;
    let suffixApplied = 0;
    let fallbackWarnings = 0;
    let photosReady = 0;
    let videosReady = 0;

    for (const plan of plans) {
      if (plan.plan_status === "ready") {
        readyOperations += 1;
        if (plan.conflict_status === "suffix_applied") {
          suffixApplied += 1;
        }

        const metadata = metadataByFileId[plan.file_id];
        if (metadata?.date_kind === "fallback") {
          fallbackWarnings += 1;
        }

        const row = rowsById.get(plan.file_id);
        if (row?.kind === "photo") {
          photosReady += 1;
        } else if (row?.kind === "video") {
          videosReady += 1;
        }
        continue;
      }

      if (plan.plan_status === "blocked_no_date") {
        blockedNoDate += 1;
        continue;
      }

      if (plan.plan_status === "blocked_conflict") {
        blockedConflict += 1;
        continue;
      }

      if (plan.plan_status === "error") {
        planErrorsCount += 1;
      }
    }

    return {
      total_plans: plans.length,
      ready_operations: readyOperations,
      skipped_operations: plans.length - readyOperations,
      blocked_no_date: blockedNoDate,
      blocked_conflict: blockedConflict,
      plan_errors: planErrorsCount,
      suffix_applied: suffixApplied,
      fallback_warnings: fallbackWarnings,
      photos_ready: photosReady,
      videos_ready: videosReady
    };
  }, [metadataByFileId, movePlansByFileId, rows]);

  const requestMovePlansForMetadata = useCallback(
    (rows: FileRecord[], metadataResults: MetadataResult[]) => {
      if (!selectedFolder) {
        return;
      }

      const rowsById = new Map(rows.map((row) => [row.id, row]));
      const filesToPlan: FileRecord[] = [];
      const metadataToPlan: MetadataResult[] = [];
      const previousSignatures = new Map<string, string | undefined>();

      for (const metadata of metadataResults) {
        const row = rowsById.get(metadata.file_id);
        if (!row || (row.kind !== "photo" && row.kind !== "video")) {
          continue;
        }

        const nextSignature = metadataPlanningSignature(metadata);
        const previousSignature = plannedMetadataSignatureByFileIdRef.current.get(row.id);
        if (previousSignature === nextSignature) {
          continue;
        }

        previousSignatures.set(row.id, previousSignature);
        plannedMetadataSignatureByFileIdRef.current.set(row.id, nextSignature);
        filesToPlan.push(row);
        metadataToPlan.push(metadata);
      }

      if (filesToPlan.length === 0) {
        return;
      }

      activePlanRequestsRef.current += 1;
      setActivePlanRequests(activePlanRequestsRef.current);

      void createMovePlan(selectedFolder, filesToPlan, metadataToPlan).catch((error: unknown) => {
        activePlanRequestsRef.current = Math.max(0, activePlanRequestsRef.current - 1);
        setActivePlanRequests(activePlanRequestsRef.current);
        for (const [fileId, previousSignature] of previousSignatures) {
          if (previousSignature === undefined) {
            plannedMetadataSignatureByFileIdRef.current.delete(fileId);
          } else {
            plannedMetadataSignatureByFileIdRef.current.set(fileId, previousSignature);
          }
        }
        setFolderError(`Failed to create move plan: ${String(error)}`);
      });
    },
    [selectedFolder]
  );

  const pumpMetadataQueue = useCallback(() => {
    while (
      activeMetadataRequestsRef.current < metadataConcurrency &&
      metadataQueueRef.current.length > 0
    ) {
      const batchRows: FileRecord[] = [];

      while (batchRows.length < METADATA_BATCH_SIZE && metadataQueueRef.current.length > 0) {
        const nextFileId = metadataQueueRef.current.shift();
        if (!nextFileId) {
          break;
        }

        queuedMetadataIdsRef.current.delete(nextFileId);
        const row = rowByIdRef.current.get(nextFileId);
        if (!row) {
          continue;
        }

        requestedMetadataIdsRef.current.add(nextFileId);
        batchRows.push(row);
      }

      if (batchRows.length === 0) {
        continue;
      }

      activeMetadataRequestsRef.current += 1;
      setActiveMetadataRequests(activeMetadataRequestsRef.current);

      void readMetadataForFiles(
        batchRows.map((row) => ({
          file_id: row.id,
          source_path: row.source_path,
          file_kind: row.kind
        }))
      )
        .then((results) => {
          setMetadataByFileId((prev) => {
            const next = { ...prev };
            for (const result of results) {
              next[result.file_id] = result;
            }
            return next;
          });

          for (const result of results) {
            if (result.metadata_status === "error" && result.error) {
              setFolderError(result.error);
            }
          }
          requestMovePlansForMetadata(batchRows, results);
        })
        .catch((error: unknown) => {
          setFolderError(`Failed to read metadata batch: ${String(error)}`);
        })
        .finally(() => {
          activeMetadataRequestsRef.current -= 1;
          setActiveMetadataRequests(activeMetadataRequestsRef.current);
          pumpMetadataQueueRef.current();
        });
    }
  }, [metadataConcurrency, requestMovePlansForMetadata]);

  useEffect(() => {
    pumpMetadataQueueRef.current = pumpMetadataQueue;
  }, [pumpMetadataQueue]);

  const enqueueReadyMovePlans = useCallback(() => {
    for (const plan of Object.values(movePlansByFileId)) {
      if (plan.plan_status !== "ready") {
        continue;
      }

      const fileId = plan.file_id;
      if (trackedMoveIdsRef.current.has(fileId)) {
        continue;
      }

      if (isMoveResultTerminal(moveResultsByFileId[fileId])) {
        trackedMoveIdsRef.current.add(fileId);
        completedMoveIdsRef.current.add(fileId);
        continue;
      }

      trackedMoveIdsRef.current.add(fileId);
      queuedMoveIdsRef.current.add(fileId);
      moveQueueRef.current.push(fileId);
    }
  }, [movePlansByFileId, moveResultsByFileId]);

  const pumpMoveQueue = useCallback(() => {
    if (!executionConfirmed || !selectedFolder || isMoveDispatchPaused) {
      return;
    }

    while (
      activeMoveRequestsRef.current < MOVE_DISPATCH_CONCURRENCY &&
      moveQueueRef.current.length > 0
    ) {
      const batchRows: FileRecord[] = [];
      const batchMetadata: MetadataResult[] = [];
      const batchPlans: MovePlan[] = [];
      const batchFileIds: string[] = [];

      while (moveQueueRef.current.length > 0) {
        const nextFileId = moveQueueRef.current.shift();
        if (!nextFileId) {
          continue;
        }

        queuedMoveIdsRef.current.delete(nextFileId);

        if (
          inflightMoveIdsRef.current.has(nextFileId) ||
          completedMoveIdsRef.current.has(nextFileId)
        ) {
          continue;
        }

        const row = rowByIdRef.current.get(nextFileId);
        const metadata = metadataByFileId[nextFileId];
        const plan = movePlansByFileId[nextFileId];
        if (!row || !metadata || !plan || plan.plan_status !== "ready") {
          trackedMoveIdsRef.current.delete(nextFileId);
          continue;
        }

        batchRows.push(row);
        batchMetadata.push(metadata);
        batchPlans.push(plan);
        batchFileIds.push(nextFileId);
      }

      if (batchRows.length === 0) {
        continue;
      }

      activeMoveRequestsRef.current += 1;
      for (const fileId of batchFileIds) {
        inflightMoveIdsRef.current.add(fileId);
      }
      setIsExecutingMoves(true);
      setMoveCompleted(false);

      void executeMovePlan(selectedFolder, batchRows, batchMetadata, batchPlans).catch(
        (error: unknown) => {
          activeMoveRequestsRef.current = Math.max(0, activeMoveRequestsRef.current - 1);
          for (const fileId of batchFileIds) {
            inflightMoveIdsRef.current.delete(fileId);
            trackedMoveIdsRef.current.delete(fileId);
          }
          setFolderError(`Failed to execute move plan: ${String(error)}`);
          if (activeMoveRequestsRef.current === 0 && moveQueueRef.current.length === 0) {
            setIsExecutingMoves(false);
            setMoveCompleted(true);
          }
          queueMicrotask(() => {
            pumpMoveQueueRef.current();
          });
        }
      );
    }
  }, [
    executionConfirmed,
    isMoveDispatchPaused,
    metadataByFileId,
    movePlansByFileId,
    selectedFolder
  ]);

  useEffect(() => {
    pumpMoveQueueRef.current = pumpMoveQueue;
  }, [pumpMoveQueue]);

  const replanForCurrentDestinationSettings = useCallback(() => {
    if (!selectedFolder || isExecutingMoves || executionConfirmed) {
      return;
    }

    const metadataResults = rows
      .map((row) => metadataByFileId[row.id])
      .filter((metadata): metadata is MetadataResult => metadata !== undefined);
    if (metadataResults.length === 0) {
      return;
    }

    plannedMetadataSignatureByFileIdRef.current = new Map();
    setMovePlansByFileId({});
    setMoveResultsByFileId({});
    setExecutionConfirmed(false);
    setIsMoveDispatchPaused(false);
    setMoveCompleted(false);
    setMovedCount(0);
    setCopiedCount(0);
    setMoveSkippedCount(0);
    setMoveFailedCount(0);
    queuedMoveIdsRef.current = new Set();
    moveQueueRef.current = [];
    inflightMoveIdsRef.current = new Set();
    completedMoveIdsRef.current = new Set();
    trackedMoveIdsRef.current = new Set();
    activeMoveRequestsRef.current = 0;

    requestMovePlansForMetadata(rows, metadataResults);
  }, [
    executionConfirmed,
    isExecutingMoves,
    metadataByFileId,
    requestMovePlansForMetadata,
    rows,
    selectedFolder
  ]);

  const applyDestinationSettingsChange = useCallback(
    async (operation: () => Promise<DestinationSettings | null>) => {
      if (destinationSettingsLocked) {
        return;
      }

      setDestinationSettingsPending(true);
      try {
        const nextSettings = await operation();
        if (nextSettings) {
          setDestinationSettings(nextSettings);
        } else {
          setDestinationSettings(await getDestinationSettings());
        }
        setDestinationSettingsError(null);
        replanForCurrentDestinationSettings();
      } catch (error: unknown) {
        setDestinationSettingsError(`Failed to update destination settings: ${String(error)}`);
      } finally {
        setDestinationSettingsPending(false);
      }
    },
    [destinationSettingsLocked, replanForCurrentDestinationSettings]
  );

  const reloadHistoryRuns = useCallback(async (preferredRunId: string | null = null) => {
    setHistoryLoading(true);
    try {
      const runs = await listHistoryRuns();
      setHistoryRuns(runs);
      setHistoryError(null);
      setSelectedHistoryRunId((previous) => {
        const preferred =
          preferredRunId ??
          (previous && runs.some((run) => run.run_id === previous) ? previous : null);
        if (preferred) {
          return preferred;
        }
        return null;
      });
    } catch (error) {
      setHistoryError(`Failed to load history runs: ${String(error)}`);
    } finally {
      setHistoryLoading(false);
    }
  }, []);

  useEffect(() => {
    queueMicrotask(() => {
      void reloadHistoryRuns();
    });
  }, [reloadHistoryRuns]);

  useEffect(() => {
    let unsubs: (() => void)[] = [];
    let disposed = false;

    const handleHistoryProgress = (
      action: HistoryActionKind,
      payload: HistoryUndoProgressEvent | HistoryRedoProgressEvent
    ) => {
      const active = activeHistoryActionRef.current;
      if (!active || active.action !== action || active.runId !== payload.run_id) {
        return;
      }

      if (payload.operation.status === "blocked" || payload.operation.status === "failed") {
        activeHistoryActionRef.current = {
          ...active,
          blockedOrFailed: active.blockedOrFailed + 1
        };
      }

      if (payload.completed_operations < payload.total_operations) {
        return;
      }

      const completed = activeHistoryActionRef.current;
      if (completed && completed.action === action && completed.runId === payload.run_id) {
        if (completed.blockedOrFailed > 0) {
          setFolderError(
            `${action === "undo" ? "Undo" : "Redo"} could not be completed: ${completed.blockedOrFailed} operation(s) were blocked or failed.`
          );
        }
        activeHistoryActionRef.current = null;
      }
    };

    void attachHistoryListeners({
      onIndexUpdated: ({ run_id }) => {
        void reloadHistoryRuns(run_id);
      },
      onUndoProgress: (payload) => {
        handleHistoryProgress("undo", payload);
      },
      onRedoProgress: (payload) => {
        handleHistoryProgress("redo", payload);
      }
    })
      .then((listeners) => {
        if (disposed) {
          for (const unlisten of listeners) {
            unlisten();
          }
          return;
        }
        unsubs = listeners;
      })
      .catch((error: unknown) => {
        if (!disposed) {
          setHistoryError(`Failed to attach history listeners: ${String(error)}`);
        }
      });

    return () => {
      disposed = true;
      for (const unlisten of unsubs) {
        unlisten();
      }
    };
  }, [reloadHistoryRuns]);

  useEffect(() => {
    if (!selectedHistoryRunId) {
      return;
    }

    let ignore = false;
    queueMicrotask(() => {
      if (!ignore) {
        setHistoryDetailsLoading(true);
      }
    });
    void getHistoryRun(selectedHistoryRunId)
      .then((details) => {
        if (ignore) {
          return;
        }
        setSelectedHistoryRunDetails(details);
      })
      .catch((error: unknown) => {
        if (!ignore) {
          setHistoryError(`Failed to load history details: ${String(error)}`);
        }
      })
      .finally(() => {
        if (!ignore) {
          setHistoryDetailsLoading(false);
        }
      });

    return () => {
      ignore = true;
    };
  }, [selectedHistoryRunId]);

  useEffect(() => {
    let ignore = false;
    Promise.all([healthCheck(), getProcessingConfig(), getDestinationSettings()])
      .then(([value, config, settings]) => {
        if (ignore) {
          return;
        }
        setHealth(value);
        setMetadataConcurrency(config.metadata_concurrency);
        setMetadataQueueCapacity(config.metadata_queue_capacity);
        setDestinationSettings(settings);
        setDestinationSettingsError(null);
      })
      .catch((error: unknown) => {
        if (!ignore) {
          setHealth(`error: ${String(error)}`);
          setDestinationSettingsError(`Failed to load destination settings: ${String(error)}`);
        }
      })
      .finally(() => {
        if (!ignore) {
          setDestinationSettingsLoading(false);
        }
      });
    return () => {
      ignore = true;
    };
  }, []);

  useEffect(() => {
    let unsubs: (() => void)[] = [];
    let disposed = false;
    void attachScanListeners({
      onStarted: () => {
        setIsScanning(true);
        setCompleted(false);
        setCancelled(false);
        setDiscovered(0);
        setSkipped(0);
        setErrors(0);
        setMetadataByFileId({});
        setMovePlansByFileId({});
        setMoveResultsByFileId({});
        setActiveMetadataRequests(0);
        setActivePlanRequests(0);
        setPlanErrors(0);
        setExecutionConfirmed(false);
        setIsMoveDispatchPaused(false);
        setIsExecutingMoves(false);
        setMoveCompleted(false);
        setMovedCount(0);
        setCopiedCount(0);
        setMoveSkippedCount(0);
        setMoveFailedCount(0);
        requestedMetadataIdsRef.current = new Set();
        queuedMetadataIdsRef.current = new Set();
        metadataQueueRef.current = [];
        activeMetadataRequestsRef.current = 0;
        plannedMetadataSignatureByFileIdRef.current = new Map();
        activePlanRequestsRef.current = 0;
        queuedMoveIdsRef.current = new Set();
        moveQueueRef.current = [];
        inflightMoveIdsRef.current = new Set();
        completedMoveIdsRef.current = new Set();
        trackedMoveIdsRef.current = new Set();
        activeMoveRequestsRef.current = 0;
        seenRowIdsRef.current = new Set();
        rowByIdRef.current = new Map();
        setRows([]);
      },
      onFile: ({ record }) => {
        if (seenRowIdsRef.current.has(record.id)) {
          return;
        }
        seenRowIdsRef.current.add(record.id);
        setDiscovered((count) => count + 1);
        setRows((prev) => [...prev, record]);
      },
      onSkipped: ({ record }) => {
        if (seenRowIdsRef.current.has(record.id)) {
          return;
        }
        seenRowIdsRef.current.add(record.id);
        setSkipped((count) => count + 1);
        setRows((prev) => [...prev, record]);
      },
      onError: () => {
        setErrors((count) => count + 1);
      },
      onFinished: (payload: ScanFinishedEvent) => {
        setIsScanning(false);
        setCompleted(!payload.cancelled);
        setCancelled(payload.cancelled);
      }
    })
      .then((listeners) => {
        if (disposed) {
          for (const unlisten of listeners) {
            unlisten();
          }
          return;
        }
        unsubs = listeners;
      })
      .catch((error: unknown) => {
        if (!disposed) {
          setFolderError(`Failed to attach scan listeners: ${String(error)}`);
        }
      });

    return () => {
      disposed = true;
      for (const unlisten of unsubs) {
        unlisten();
      }
    };
  }, []);

  useEffect(() => {
    let unsubs: (() => void)[] = [];
    let disposed = false;
    void attachMoveListeners({
      onFile: ({ result }) => {
        if (completedMoveIdsRef.current.has(result.file_id)) {
          setMoveResultsByFileId((prev) => ({ ...prev, [result.file_id]: result }));
          return;
        }
        setMoveResultsByFileId((prev) => ({ ...prev, [result.file_id]: result }));
        inflightMoveIdsRef.current.delete(result.file_id);
        completedMoveIdsRef.current.add(result.file_id);
        trackedMoveIdsRef.current.add(result.file_id);
        if (result.result_status === "moved") {
          setMovedCount((count) => count + 1);
          return;
        }
        if (result.result_status === "copied") {
          setCopiedCount((count) => count + 1);
          return;
        }
        if (result.result_status === "skipped") {
          setMoveSkippedCount((count) => count + 1);
          return;
        }
        setMoveFailedCount((count) => count + 1);
        if (result.error) {
          setFolderError(`Move error: ${result.error}`);
        }
      },
      onFinished: () => {
        activeMoveRequestsRef.current = Math.max(0, activeMoveRequestsRef.current - 1);
        if (activeMoveRequestsRef.current === 0 && moveQueueRef.current.length === 0) {
          setIsExecutingMoves(false);
          setMoveCompleted(true);
        }
        queueMicrotask(() => {
          pumpMoveQueueRef.current();
        });
      }
    })
      .then((listeners) => {
        if (disposed) {
          for (const unlisten of listeners) {
            unlisten();
          }
          return;
        }
        unsubs = listeners;
      })
      .catch((error: unknown) => {
        if (!disposed) {
          setFolderError(`Failed to attach move listeners: ${String(error)}`);
        }
      });

    return () => {
      disposed = true;
      for (const unlisten of unsubs) {
        unlisten();
      }
    };
  }, []);

  useEffect(() => {
    let unsubs: (() => void)[] = [];
    let disposed = false;
    void attachPlanListeners({
      onFile: ({ plan }) => {
        setMovePlansByFileId((prev) => ({ ...prev, [plan.file_id]: plan }));
      },
      onError: ({ error }) => {
        setPlanErrors((count) => count + 1);
        setFolderError(`Plan error: ${error}`);
      },
      onFinished: () => {
        activePlanRequestsRef.current = Math.max(0, activePlanRequestsRef.current - 1);
        setActivePlanRequests(activePlanRequestsRef.current);
      }
    })
      .then((listeners) => {
        if (disposed) {
          for (const unlisten of listeners) {
            unlisten();
          }
          return;
        }
        unsubs = listeners;
      })
      .catch((error: unknown) => {
        if (!disposed) {
          setFolderError(`Failed to attach plan listeners: ${String(error)}`);
        }
      });

    return () => {
      disposed = true;
      for (const unlisten of unsubs) {
        unlisten();
      }
    };
  }, []);

  useEffect(() => {
    rowByIdRef.current = new Map(rows.map((row) => [row.id, row]));

    for (const row of rows) {
      if (row.kind !== "photo" && row.kind !== "video") {
        continue;
      }

      if (metadataByFileId[row.id]) {
        continue;
      }

      if (requestedMetadataIdsRef.current.has(row.id) || queuedMetadataIdsRef.current.has(row.id)) {
        continue;
      }

      if (metadataQueueRef.current.length >= metadataQueueCapacity) {
        break;
      }

      metadataQueueRef.current.push(row.id);
      queuedMetadataIdsRef.current.add(row.id);
    }

    pumpMetadataQueue();
  }, [metadataByFileId, metadataQueueCapacity, pumpMetadataQueue, rows]);

  const handleChooseRootMediaFolder = async () => {
    await applyDestinationSettingsChange(async () => chooseRootMediaFolder());
  };

  const handleClearRootMediaFolder = async () => {
    await applyDestinationSettingsChange(async () => clearRootMediaFolder());
  };

  const handleSetForceProcessedFolder = async (nextValue: boolean) => {
    await applyDestinationSettingsChange(async () => setForceProcessedFolder(nextValue));
  };

  const handleChooseFolder = async () => {
    try {
      const chosen = await chooseFolder();
      if (!chosen) {
        return;
      }
      if (isProcessedPath(chosen)) {
        setSelectedFolder(chosen);
        setFolderError("Selected folder is inside a Processed path.");
        return;
      }
      setSelectedFolder(chosen);
      setFolderError(null);
      await startScan(chosen);
    } catch (error) {
      setFolderError(`Failed to choose folder or start scan: ${String(error)}`);
      setIsScanning(false);
    }
  };

  const handleCancelScan = async () => {
    try {
      await cancelScan();
    } catch (error) {
      setFolderError(`Failed to cancel scan: ${String(error)}`);
    }
  };

  useEffect(() => {
    enqueueReadyMovePlans();
    if (executionConfirmed) {
      pumpMoveQueueRef.current();
    }
  }, [enqueueReadyMovePlans, executionConfirmed]);

  const handleConfirmExecution = () => {
    if (pendingExecutableCount === 0 || executionConfirmed) {
      return;
    }

    const message =
      `Ready operations: ${confirmationSummary.ready_operations}\n` +
      `Skipped operations: ${confirmationSummary.skipped_operations}\n` +
      `Fallback warnings: ${confirmationSummary.fallback_warnings}\n` +
      `Suffix-applied destinations: ${confirmationSummary.suffix_applied}\n\n` +
      "Proceed with move execution now?";

    if (window.confirm(message)) {
      if (!selectedFolder) {
        setFolderError("No selected folder for move execution.");
        return;
      }

      queuedMoveIdsRef.current = new Set();
      moveQueueRef.current = [];
      inflightMoveIdsRef.current = new Set();
      completedMoveIdsRef.current = new Set();
      trackedMoveIdsRef.current = new Set();
      activeMoveRequestsRef.current = 0;

      setExecutionConfirmed(true);
      setIsMoveDispatchPaused(false);
      setIsExecutingMoves(true);
      setMoveCompleted(false);
      setMovedCount(0);
      setCopiedCount(0);
      setMoveSkippedCount(0);
      setMoveFailedCount(0);
      setMoveResultsByFileId({});
      for (const plan of Object.values(movePlansByFileId)) {
        if (plan.plan_status !== "ready") {
          continue;
        }
        trackedMoveIdsRef.current.add(plan.file_id);
        queuedMoveIdsRef.current.add(plan.file_id);
        moveQueueRef.current.push(plan.file_id);
      }
      pumpMoveQueueRef.current();
    }
  };

  const handleToggleMoveDispatchPause = () => {
    if (!executionConfirmed || moveCompleted) {
      return;
    }

    setIsMoveDispatchPaused((previous) => {
      const next = !previous;
      if (!next) {
        queueMicrotask(() => {
          pumpMoveQueueRef.current();
        });
      }
      return next;
    });
  };

  const handleExecutionAction = () => {
    if (executionConfirmed) {
      handleToggleMoveDispatchPause();
      return;
    }
    handleConfirmExecution();
  };

  const handleUndoProcessing = async (runId: string) => {
    try {
      activeHistoryActionRef.current = {
        runId,
        action: "undo",
        blockedOrFailed: 0
      };
      await undoHistoryRun(runId);
      setFolderError(null);
      await reloadHistoryRuns(runId);
      setSelectedHistoryRunDetails(await getHistoryRun(runId));
    } catch (error) {
      activeHistoryActionRef.current = null;
      setFolderError(`Failed to undo processing: ${String(error)}`);
    }
  };

  const handleRedoProcessing = async (runId: string) => {
    try {
      activeHistoryActionRef.current = {
        runId,
        action: "redo",
        blockedOrFailed: 0
      };
      await redoHistoryRun(runId);
      setFolderError(null);
      await reloadHistoryRuns(runId);
      setSelectedHistoryRunDetails(await getHistoryRun(runId));
    } catch (error) {
      activeHistoryActionRef.current = null;
      setFolderError(`Failed to redo processing: ${String(error)}`);
    }
  };

  const handleDeleteProcessing = async (runId: string) => {
    try {
      await deleteHistoryRun(runId);
      setSelectedHistoryRunDetails((prev) => (prev?.run.run_id === runId ? null : prev));
      setSelectedHistoryRunId((prev) => (prev === runId ? null : prev));
      setFolderError(null);
      await reloadHistoryRuns();
    } catch (error) {
      setFolderError(`Failed to delete history entry: ${String(error)}`);
    }
  };

  const visibleHistoryDetails =
    selectedHistoryRunId && selectedHistoryRunDetails?.run.run_id === selectedHistoryRunId
      ? selectedHistoryRunDetails
      : null;

  return (
    <div className="flex h-full flex-col bg-slate-950 text-slate-100">
      <Toolbar
        selectedFolder={selectedFolder}
        folderError={folderError}
        isScanning={isScanning}
        historyRuns={historyRuns}
        selectedHistoryRunId={selectedHistoryRunId}
        historyLoading={historyLoading}
        details={visibleHistoryDetails}
        detailsLoading={historyDetailsLoading}
        detailsError={historyError}
        rootMediaFolder={destinationSettings?.root_media_folder ?? null}
        settingsMenu={
          <DestinationSettingsMenu
            settings={destinationSettings}
            loading={destinationSettingsLoading}
            pending={destinationSettingsPending}
            locked={destinationSettingsLocked}
            error={destinationSettingsError}
            onChooseRoot={() => void handleChooseRootMediaFolder()}
            onClearRoot={() => void handleClearRootMediaFolder()}
            onSetForceProcessed={(nextValue) => void handleSetForceProcessedFolder(nextValue)}
          />
        }
        onChooseFolder={() => void handleChooseFolder()}
        onCancelScan={() => void handleCancelScan()}
        onSelectHistoryRun={(runId) => {
          if (runId !== selectedHistoryRunId) {
            setSelectedHistoryRunId(runId);
            setSelectedHistoryRunDetails(null);
          }
        }}
        onUndoProcessing={(runId) => void handleUndoProcessing(runId)}
        onRedoProcessing={(runId) => void handleRedoProcessing(runId)}
        onDeleteProcessing={(runId) => void handleDeleteProcessing(runId)}
      />

      <main className="min-h-0 flex-1">
        <FileTable
          rows={rows}
          metadataByFileId={metadataByFileId}
          movePlansByFileId={movePlansByFileId}
          moveResultsByFileId={moveResultsByFileId}
          rootMediaFolder={destinationSettings?.root_media_folder ?? null}
        />
      </main>

      <section className="border-t border-slate-800 bg-slate-900 px-3 py-2 text-xs text-slate-300">
        <div className="flex flex-wrap items-center gap-3">
          <span className="font-semibold text-slate-200">Execution Summary</span>
          {confirmationSummary.total_plans > 0 ? (
            <>
              <span>Ready: {confirmationSummary.ready_operations}</span>
              <span>Skipped: {confirmationSummary.skipped_operations}</span>
              <span>Warnings: {confirmationSummary.fallback_warnings}</span>
              <span>Suffixes: {confirmationSummary.suffix_applied}</span>
            </>
          ) : (
            <span className="text-slate-500">No move summary yet</span>
          )}
          <Button
            variant="secondary"
            className="ml-auto bg-amber-600 px-3 py-1.5 text-xs text-white hover:bg-amber-500"
            disabled={moveCompleted || !hasPendingExecutableItems}
            onClick={handleExecutionAction}
          >
            {moveCompleted
              ? "Complete"
              : !executionConfirmed
                ? "Confirm & Execute"
                : isMoveDispatchPaused
                  ? "Resume Dispatch"
                  : "Pause Dispatch"}
          </Button>
        </div>
        {executionConfirmed && !moveCompleted ? (
          <p className={`mt-1 ${isMoveDispatchPaused ? "text-amber-400" : "text-emerald-400"}`}>
            {isMoveDispatchPaused
              ? "Move dispatch paused. In-flight jobs may still complete."
              : "Execution explicitly confirmed."}
          </p>
        ) : null}
      </section>

      <StatusBar
        discovered={discovered}
        skipped={skipped}
        errors={errors}
        planErrors={planErrors}
        isPlanning={isPlanning}
        planCompleted={planCompleted}
        completed={completed}
        cancelled={cancelled}
        isExecutingMoves={isExecutingMoves}
        moveCompleted={moveCompleted}
        moved={movedCount}
        copied={copiedCount}
        moveSkipped={moveSkippedCount}
        moveFailed={moveFailedCount}
      />

      <div className="border-t border-slate-800 px-3 py-1 text-xs text-slate-500">
        Backend: {health}
      </div>
    </div>
  );
}
