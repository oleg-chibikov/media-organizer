import { useEffect, useRef, useState } from "react";
import { FileTable } from "./components/FileTable";
import { StatusBar } from "./components/StatusBar";
import { Toolbar } from "./components/Toolbar";
import {
  attachPlanListeners,
  attachScanListeners,
  cancelScan,
  chooseFolder,
  createMovePlan,
  healthCheck,
  readMetadataForFile,
  startScan
} from "./lib/api";
import type {
  FileRecord,
  MetadataResult,
  MovePlan,
  ScanFinishedEvent
} from "./lib/types";

function isProcessedPath(folder: string): boolean {
  return folder
    .toLowerCase()
    .split(/[/\\]+/)
    .some((segment) => segment === "processed");
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
  const [metadataByFileId, setMetadataByFileId] = useState<
    Record<string, MetadataResult>
  >({});
  const [movePlansByFileId, setMovePlansByFileId] = useState<
    Record<string, MovePlan>
  >({});
  const [isLoadingMetadata, setIsLoadingMetadata] = useState(false);
  const [isPlanning, setIsPlanning] = useState(false);
  const [planErrors, setPlanErrors] = useState(0);
  const [planCompleted, setPlanCompleted] = useState(false);
  const seenRowIdsRef = useRef<Set<string>>(new Set());
  const requestedMetadataIdsRef = useRef<Set<string>>(new Set());
  const lastPlanRequestKeyRef = useRef<string>("");

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
        setIsLoadingMetadata(false);
        setIsPlanning(false);
        setPlanErrors(0);
        setPlanCompleted(false);
        lastPlanRequestKeyRef.current = "";
        requestedMetadataIdsRef.current = new Set();
        seenRowIdsRef.current = new Set();
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
    void attachPlanListeners({
      onFile: ({ plan }) => {
        setMovePlansByFileId((prev) => ({ ...prev, [plan.file_id]: plan }));
      },
      onError: ({ error }) => {
        setPlanErrors((count) => count + 1);
        setFolderError(`Plan error: ${error}`);
      },
      onFinished: () => {
        setIsPlanning(false);
        setPlanCompleted(true);
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

  const loadMetadataForRow = async (row: FileRecord) => {
    setIsLoadingMetadata(true);
    try {
      const result = await readMetadataForFile(row.id, row.source_path, row.kind);
      setMetadataByFileId((prev) => ({ ...prev, [result.file_id]: result }));
      if (result.metadata_status === "error" && result.error) {
        setFolderError(result.error);
      }
    } catch (error) {
      setFolderError(`Failed to read metadata: ${String(error)}`);
    } finally {
      setIsLoadingMetadata(false);
    }
  };

  useEffect(() => {
    if (isScanning || rows.length === 0 || isLoadingMetadata) {
      return;
    }

    const firstMediaRow = rows.find(
      (row) =>
        (row.kind === "photo" || row.kind === "video") &&
        !metadataByFileId[row.id] &&
        !requestedMetadataIdsRef.current.has(row.id)
    );
    if (!firstMediaRow) {
      return;
    }
    requestedMetadataIdsRef.current.add(firstMediaRow.id);
    void loadMetadataForRow(firstMediaRow);
  }, [isScanning, isLoadingMetadata, metadataByFileId, rows]);

  useEffect(() => {
    if (isScanning || isLoadingMetadata || !selectedFolder || rows.length === 0) {
      return;
    }

    const mediaRows = rows.filter(
      (row) => row.kind === "photo" || row.kind === "video"
    );
    if (mediaRows.length === 0) {
      return;
    }

    const allMediaResolved = mediaRows.every((row) => Boolean(metadataByFileId[row.id]));
    if (!allMediaResolved) {
      return;
    }

    const requestKey = mediaRows
      .map((row) => {
        const metadata = metadataByFileId[row.id];
        return `${row.id}:${metadata.metadata_status}:${metadata.chosen_date ?? ""}`;
      })
      .join("|");

    if (requestKey === lastPlanRequestKeyRef.current) {
      return;
    }
    lastPlanRequestKeyRef.current = requestKey;

    setIsPlanning(true);
    setPlanCompleted(false);
    setPlanErrors(0);
    setMovePlansByFileId({});
    void createMovePlan(selectedFolder, rows, Object.values(metadataByFileId)).catch(
      (error: unknown) => {
        setIsPlanning(false);
        setFolderError(`Failed to create move plan: ${String(error)}`);
      }
    );
  }, [isScanning, isLoadingMetadata, metadataByFileId, rows, selectedFolder]);

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
        />
      </main>

      <StatusBar
        discovered={discovered}
        skipped={skipped}
        errors={errors}
        planErrors={planErrors}
        isPlanning={isPlanning}
        planCompleted={planCompleted}
        completed={completed}
        cancelled={cancelled}
      />

      <div className="border-t border-slate-800 px-3 py-1 text-xs text-slate-500">
        Backend: {health}
      </div>
    </div>
  );
}
