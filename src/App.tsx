import { useCallback, useEffect, useRef, useState } from "react";
import { FileTable } from "./components/FileTable";
import { StatusBar } from "./components/StatusBar";
import { Toolbar } from "./components/Toolbar";
import {
  attachMoveListeners,
  attachPlanListeners,
  attachScanListeners,
  cancelScan,
  chooseFolder,
  createMovePlan,
  executeMovePlan,
  getMoveConfirmationSummary,
  healthCheck,
  readMetadataForFile,
  startScan
} from "./lib/api";
import type {
  FileRecord,
  MetadataResult,
  MoveConfirmationSummary,
  MovePlan,
  MoveResult,
  ScanFinishedEvent
} from "./lib/types";

const METADATA_CONCURRENCY = 2;
const METADATA_QUEUE_CAPACITY = 256;

function isProcessedPath(folder: string): boolean {
  return folder
    .toLowerCase()
    .split(/[/\\]+/)
    .some((segment) => segment === "processed");
}

function metadataPlanningSignature(metadata: MetadataResult): string {
  return `${metadata.metadata_status}:${metadata.chosen_date ?? ""}`;
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
  const [isPlanning, setIsPlanning] = useState(false);
  const [planErrors, setPlanErrors] = useState(0);
  const [planCompleted, setPlanCompleted] = useState(false);
  const [confirmationSummary, setConfirmationSummary] = useState<MoveConfirmationSummary | null>(
    null
  );
  const [isBuildingConfirmation, setIsBuildingConfirmation] = useState(false);
  const [executionConfirmed, setExecutionConfirmed] = useState(false);
  const [isExecutingMoves, setIsExecutingMoves] = useState(false);
  const [moveCompleted, setMoveCompleted] = useState(false);
  const [movedCount, setMovedCount] = useState(0);
  const [moveSkippedCount, setMoveSkippedCount] = useState(0);
  const [moveFailedCount, setMoveFailedCount] = useState(0);

  const seenRowIdsRef = useRef<Set<string>>(new Set());
  const requestedMetadataIdsRef = useRef<Set<string>>(new Set());
  const queuedMetadataIdsRef = useRef<Set<string>>(new Set());
  const metadataQueueRef = useRef<string[]>([]);
  const activeMetadataRequestsRef = useRef(0);
  const rowByIdRef = useRef<Map<string, FileRecord>>(new Map());
  const pumpMetadataQueueRef = useRef<() => void>(() => undefined);
  const plannedMetadataSignatureByFileIdRef = useRef<Map<string, string>>(new Map());
  const activePlanRequestsRef = useRef(0);
  const lastConfirmationRequestKeyRef = useRef<string>("");
  const pendingExecutableCount = Object.values(movePlansByFileId).filter((plan) => {
    if (plan.plan_status !== "ready") {
      return false;
    }
    return moveResultsByFileId[plan.file_id]?.result_status !== "moved";
  }).length;

  const pumpMetadataQueue = useCallback(() => {
    while (
      activeMetadataRequestsRef.current < METADATA_CONCURRENCY &&
      metadataQueueRef.current.length > 0
    ) {
      const nextFileId = metadataQueueRef.current.shift();
      if (!nextFileId) {
        break;
      }

      queuedMetadataIdsRef.current.delete(nextFileId);
      const row = rowByIdRef.current.get(nextFileId);
      if (!row) {
        continue;
      }

      activeMetadataRequestsRef.current += 1;
      requestedMetadataIdsRef.current.add(nextFileId);

      void readMetadataForFile(row.id, row.source_path, row.kind)
        .then((result) => {
          setMetadataByFileId((prev) => ({ ...prev, [result.file_id]: result }));
          if (result.metadata_status === "error" && result.error) {
            setFolderError(result.error);
          }
        })
        .catch((error: unknown) => {
          setFolderError(`Failed to read metadata: ${String(error)}`);
        })
        .finally(() => {
          activeMetadataRequestsRef.current -= 1;
          pumpMetadataQueueRef.current();
        });
    }
  }, []);

  useEffect(() => {
    pumpMetadataQueueRef.current = pumpMetadataQueue;
  }, [pumpMetadataQueue]);

  useEffect(() => {
    let ignore = false;
    healthCheck()
      .then((value) => {
        if (!ignore) setHealth(value);
      })
      .catch((error: unknown) => {
        if (!ignore) setHealth(`error: ${String(error)}`);
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
        setIsPlanning(false);
        setPlanErrors(0);
        setPlanCompleted(false);
        setConfirmationSummary(null);
        setIsBuildingConfirmation(false);
        setExecutionConfirmed(false);
        setIsExecutingMoves(false);
        setMoveCompleted(false);
        setMovedCount(0);
        setMoveSkippedCount(0);
        setMoveFailedCount(0);
        lastConfirmationRequestKeyRef.current = "";
        requestedMetadataIdsRef.current = new Set();
        queuedMetadataIdsRef.current = new Set();
        metadataQueueRef.current = [];
        activeMetadataRequestsRef.current = 0;
        plannedMetadataSignatureByFileIdRef.current = new Map();
        activePlanRequestsRef.current = 0;
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
        setMoveResultsByFileId((prev) => ({ ...prev, [result.file_id]: result }));
        if (result.result_status === "moved") {
          setMovedCount((count) => count + 1);
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
      onFinished: ({ moved, skipped, failed }) => {
        setIsExecutingMoves(false);
        setMoveCompleted(true);
        setMovedCount(moved);
        setMoveSkippedCount(skipped);
        setMoveFailedCount(failed);
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
        if (activePlanRequestsRef.current === 0) {
          setIsPlanning(false);
          setPlanCompleted(true);
        }
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

      if (metadataQueueRef.current.length >= METADATA_QUEUE_CAPACITY) {
        setFolderError(
          `Metadata queue reached capacity (${METADATA_QUEUE_CAPACITY}). Remaining files will wait.`
        );
        break;
      }

      metadataQueueRef.current.push(row.id);
      queuedMetadataIdsRef.current.add(row.id);
    }

    pumpMetadataQueue();
  }, [metadataByFileId, pumpMetadataQueue, rows]);

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
    if (!selectedFolder || rows.length === 0) {
      return;
    }

    const filesToPlan: FileRecord[] = [];
    const metadataToPlan: MetadataResult[] = [];
    const previousSignatures = new Map<string, string | undefined>();

    for (const row of rows) {
      if (row.kind !== "photo" && row.kind !== "video") {
        continue;
      }

      const metadata = metadataByFileId[row.id];
      if (!metadata) {
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
    queueMicrotask(() => {
      setIsPlanning(true);
      setPlanCompleted(false);
    });

    void createMovePlan(selectedFolder, filesToPlan, metadataToPlan).catch((error: unknown) => {
      activePlanRequestsRef.current = Math.max(0, activePlanRequestsRef.current - 1);
      if (activePlanRequestsRef.current === 0) {
        setIsPlanning(false);
        setPlanCompleted(true);
      }
      for (const [fileId, previousSignature] of previousSignatures) {
        if (previousSignature === undefined) {
          plannedMetadataSignatureByFileIdRef.current.delete(fileId);
        } else {
          plannedMetadataSignatureByFileIdRef.current.set(fileId, previousSignature);
        }
      }
      setFolderError(`Failed to create move plan: ${String(error)}`);
    });
  }, [metadataByFileId, rows, selectedFolder]);

  useEffect(() => {
    if (!planCompleted || isPlanning || rows.length === 0) {
      return;
    }

    const plans = Object.values(movePlansByFileId);
    if (plans.length === 0) {
      return;
    }

    const requestKey = plans
      .map((plan) => `${plan.file_id}:${plan.plan_status}:${plan.conflict_status}`)
      .join("|");

    if (requestKey === lastConfirmationRequestKeyRef.current) {
      return;
    }
    lastConfirmationRequestKeyRef.current = requestKey;

    setIsBuildingConfirmation(true);
    setExecutionConfirmed(false);
    void getMoveConfirmationSummary(rows, Object.values(metadataByFileId), plans)
      .then((summary) => {
        setConfirmationSummary(summary);
      })
      .catch((error: unknown) => {
        setConfirmationSummary(null);
        setFolderError(`Failed to build confirmation summary: ${String(error)}`);
      })
      .finally(() => {
        setIsBuildingConfirmation(false);
      });
  }, [isPlanning, metadataByFileId, movePlansByFileId, planCompleted, rows]);

  const handleConfirmExecution = () => {
    if (!confirmationSummary || pendingExecutableCount === 0 || isExecutingMoves) {
      return;
    }

    const message =
      `Ready operations: ${confirmationSummary.ready_operations}\n` +
      `Skipped operations: ${confirmationSummary.skipped_operations}\n` +
      `Fallback warnings: ${confirmationSummary.fallback_warnings}\n` +
      `Suffix-applied destinations: ${confirmationSummary.suffix_applied}\n\n` +
      "Proceed with move execution now?";

    if (window.confirm(message)) {
      setExecutionConfirmed(true);
      setIsExecutingMoves(true);
      setMoveCompleted(false);
      setMovedCount(0);
      setMoveSkippedCount(0);
      setMoveFailedCount(0);
      setMoveResultsByFileId({});
      if (!selectedFolder) {
        setIsExecutingMoves(false);
        setFolderError("No selected folder for move execution.");
        return;
      }
      void executeMovePlan(
        selectedFolder,
        rows,
        Object.values(metadataByFileId),
        Object.values(movePlansByFileId)
      ).catch((error: unknown) => {
        setIsExecutingMoves(false);
        setFolderError(`Failed to execute move plan: ${String(error)}`);
      });
    }
  };

  return (
    <div className="flex h-full flex-col bg-slate-950 text-slate-100">
      <Toolbar
        selectedFolder={selectedFolder}
        folderError={folderError}
        isScanning={isScanning}
        onChooseFolder={() => void handleChooseFolder()}
        onCancelScan={() => void handleCancelScan()}
      />

      <main className="min-h-0 flex-1">
        <FileTable
          rows={rows}
          metadataByFileId={metadataByFileId}
          movePlansByFileId={movePlansByFileId}
          moveResultsByFileId={moveResultsByFileId}
          selectedFolder={selectedFolder}
        />
      </main>

      <section className="border-t border-slate-800 bg-slate-900 px-3 py-2 text-xs text-slate-300">
        <div className="flex flex-wrap items-center gap-3">
          <span className="font-semibold text-slate-200">Execution Summary</span>
          {isBuildingConfirmation ? (
            <span className="text-slate-400">Building summary...</span>
          ) : null}
          {confirmationSummary ? (
            <>
              <span>Ready: {confirmationSummary.ready_operations}</span>
              <span>Skipped: {confirmationSummary.skipped_operations}</span>
              <span>Warnings: {confirmationSummary.fallback_warnings}</span>
              <span>Suffixes: {confirmationSummary.suffix_applied}</span>
            </>
          ) : (
            <span className="text-slate-500">No move summary yet</span>
          )}
          <button
            className="ml-auto rounded bg-amber-600 px-3 py-1.5 text-xs font-medium text-white hover:bg-amber-500 disabled:cursor-not-allowed disabled:bg-slate-700"
            disabled={
              !confirmationSummary ||
              pendingExecutableCount === 0 ||
              isBuildingConfirmation ||
              isExecutingMoves
            }
            onClick={handleConfirmExecution}
            type="button"
          >
            {isExecutingMoves ? "Executing..." : "Confirm & Execute"}
          </button>
        </div>
        {executionConfirmed ? (
          <p className="mt-1 text-emerald-400">Execution explicitly confirmed.</p>
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
        moveSkipped={moveSkippedCount}
        moveFailed={moveFailedCount}
      />

      <div className="border-t border-slate-800 px-3 py-1 text-xs text-slate-500">
        Backend: {health}
      </div>
    </div>
  );
}
