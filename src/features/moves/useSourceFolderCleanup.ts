import { useEffect, useRef } from "react";
import { removeEmptySourceFolders } from "../../lib/api";

// Runs once per finished run: batches overlap and the scan can still be walking folders before then.
export function useSourceFolderCleanup(
  completed: boolean,
  selectedFolder: string | null,
  reportError: (message: string) => void
) {
  const wasCompletedRef = useRef(completed);
  useEffect(() => {
    const justCompleted = completed && !wasCompletedRef.current;
    wasCompletedRef.current = completed;
    if (!justCompleted || !selectedFolder) {
      return;
    }
    void removeEmptySourceFolders(selectedFolder).catch((error: unknown) =>
      reportError(`Failed to remove empty source folders: ${String(error)}`)
    );
  }, [completed, selectedFolder, reportError]);
}
