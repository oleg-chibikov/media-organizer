import { useEffect, useRef, useState } from "react";
import { cancelScan, chooseFolder, startScan, type ScanListeners } from "../../lib/api";
import type { FileRecord } from "../../lib/types";
import { isProcessedPath } from "./isProcessedPath";
import { EMPTY_SCAN_PROGRESS, type ScanProgress } from "./scanProgress";
import { useReplacedScanFilter } from "./useReplacedScanFilter";
import { useScanRows } from "./useScanRows";

interface ScanSessionOptions {
  reportError: (message: string | null) => void;
  // Later steps drop what they built from the old folder's files.
  onNewScan: () => void;
}

export function useScanSession({ reportError, onNewScan }: ScanSessionOptions) {
  const { rows, rowByIdRef, addRow, removeRows, flushRows, resetRows } = useScanRows();
  const [selectedFolder, setSelectedFolder] = useState<string | null>(null);
  const [isScanning, setIsScanning] = useState(false);
  const [progress, setProgress] = useState<ScanProgress>(EMPTY_SCAN_PROGRESS);
  const replacedScan = useReplacedScanFilter();
  const onNewScanRef = useRef(onNewScan);
  useEffect(() => {
    onNewScanRef.current = onNewScan;
  });

  const clearSession = () => {
    resetRows();
    setProgress(EMPTY_SCAN_PROGRESS);
    onNewScanRef.current();
  };

  const chooseFolderAndScan = async () => {
    let chosen: string | null = null;
    try {
      chosen = await chooseFolder();
      if (!chosen) {
        return;
      }
      if (isProcessedPath(chosen)) {
        reportError("Selected folder is inside a Processed path.");
        return;
      }
      reportError(null);
      // The old folder's files leave the screen before the backend starts on the new one.
      replacedScan.awaitScan(chosen);
      clearSession();
      setSelectedFolder(chosen);
      setIsScanning(true);
      await startScan(chosen);
    } catch (error) {
      reportError(`Failed to choose folder or start scan: ${String(error)}`);
      if (chosen !== null && replacedScan.abandonScan(chosen)) {
        setSelectedFolder(null);
        setIsScanning(false);
      }
    }
  };

  const stopScan = async () => {
    // The Stop button goes at once; the finished event confirms the totals.
    setIsScanning(false);
    setProgress((prev) => ({ ...prev, cancelled: true }));
    try {
      await cancelScan();
    } catch (error) {
      reportError(`Failed to cancel scan: ${String(error)}`);
      setIsScanning(true);
      setProgress((prev) => ({ ...prev, cancelled: false }));
    }
  };

  const { acceptsStart, isReplaced } = replacedScan;
  const events: ScanListeners = {
    onStarted: ({ root }) => {
      if (acceptsStart(root)) {
        setIsScanning(true);
        clearSession();
      }
    },
    onFile: ({ record }) => {
      if (!isReplaced() && addRow(record)) {
        setProgress((prev) => ({ ...prev, discovered: prev.discovered + 1 }));
      }
    },
    onSkipped: ({ record }) => {
      if (!isReplaced() && addRow(record)) {
        setProgress((prev) => ({ ...prev, skipped: prev.skipped + 1 }));
      }
    },
    onError: () => {
      if (!isReplaced()) {
        setProgress((prev) => ({ ...prev, errors: prev.errors + 1 }));
      }
    },
    onFinished: ({ cancelled }) => {
      if (isReplaced()) {
        return;
      }
      // Rows land before the finished flag so planning never looks complete with rows still pending.
      flushRows();
      setIsScanning(false);
      setProgress((prev) => ({ ...prev, completed: !cancelled, cancelled }));
    }
  };

  return {
    rows,
    rowByIdRef,
    // Placed files from an earlier run join the scanned rows when they move again.
    addRows: (records: FileRecord[]) => records.forEach(addRow),
    removeRows,
    selectedFolder,
    isScanning,
    progress,
    events,
    chooseFolderAndScan,
    stopScan
  };
}
