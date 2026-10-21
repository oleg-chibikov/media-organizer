import { useState } from "react";
import { cancelScan, chooseFolder, startScan, type ScanListeners } from "../../lib/api";
import { isProcessedPath } from "./isProcessedPath";
import { EMPTY_SCAN_PROGRESS, type ScanProgress } from "./scanProgress";
import { useScanRows } from "./useScanRows";

export function useScanSession(reportError: (message: string | null) => void) {
  const { rows, rowByIdRef, addRow, resetRows } = useScanRows();
  const [selectedFolder, setSelectedFolder] = useState<string | null>(null);
  const [isScanning, setIsScanning] = useState(false);
  const [progress, setProgress] = useState<ScanProgress>(EMPTY_SCAN_PROGRESS);

  const chooseFolderAndScan = async () => {
    try {
      const chosen = await chooseFolder();
      if (!chosen) {
        return;
      }
      setSelectedFolder(chosen);
      if (isProcessedPath(chosen)) {
        reportError("Selected folder is inside a Processed path.");
        return;
      }
      reportError(null);
      await startScan(chosen);
    } catch (error) {
      reportError(`Failed to choose folder or start scan: ${String(error)}`);
      setIsScanning(false);
    }
  };

  const stopScan = async () => {
    try {
      await cancelScan();
    } catch (error) {
      reportError(`Failed to cancel scan: ${String(error)}`);
    }
  };

  const events: ScanListeners = {
    onStarted: () => {
      setIsScanning(true);
      setProgress(EMPTY_SCAN_PROGRESS);
      resetRows();
    },
    onFile: ({ record }) => {
      if (addRow(record)) {
        setProgress((prev) => ({ ...prev, discovered: prev.discovered + 1 }));
      }
    },
    onSkipped: ({ record }) => {
      if (addRow(record)) {
        setProgress((prev) => ({ ...prev, skipped: prev.skipped + 1 }));
      }
    },
    onError: () => {
      setProgress((prev) => ({ ...prev, errors: prev.errors + 1 }));
    },
    onFinished: ({ cancelled }) => {
      setIsScanning(false);
      setProgress((prev) => ({ ...prev, completed: !cancelled, cancelled }));
    }
  };

  return {
    rows,
    rowByIdRef,
    selectedFolder,
    isScanning,
    progress,
    events,
    chooseFolderAndScan,
    stopScan
  };
}
