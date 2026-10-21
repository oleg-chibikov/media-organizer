import type { ReactNode } from "react";
import { Button } from "../../components/design-system/Button";

interface ToolbarProps {
  selectedFolder: string | null;
  folderError: string | null;
  isScanning: boolean;
  movesRunning: boolean;
  rootMediaFolder: string | null;
  historyMenu?: ReactNode;
  settingsMenu?: ReactNode;
  onChooseFolder: () => void;
  onCancelScan: () => void;
}

export function Toolbar({
  selectedFolder,
  folderError,
  isScanning,
  movesRunning,
  rootMediaFolder,
  historyMenu,
  settingsMenu,
  onChooseFolder,
  onCancelScan
}: ToolbarProps) {
  return (
    <header className="flex flex-wrap items-center gap-2 border-b border-slate-800 bg-slate-900 px-3 py-2">
      {/* A new scan resets the move state while the backend is still moving files. */}
      <Button
        variant="primary"
        className="px-3 py-1.5 text-sm"
        disabled={movesRunning}
        onClick={onChooseFolder}
      >
        Choose Folder
      </Button>
      {isScanning ? (
        <Button
          variant="secondary"
          className="bg-rose-600 px-3 py-1.5 text-sm text-white hover:bg-rose-500"
          onClick={onCancelScan}
        >
          Stop Scan
        </Button>
      ) : null}
      {historyMenu}

      <div className="ml-2 min-w-0 flex-1 text-sm text-slate-300">
        {selectedFolder ? (
          <p className="truncate">{selectedFolder}</p>
        ) : (
          <p className="text-slate-400">No folder selected</p>
        )}
        {folderError ? <p className="text-rose-400">{folderError}</p> : null}
      </div>

      <div className="ml-auto flex min-w-0 items-center gap-2">
        {rootMediaFolder ? (
          <p className="max-w-[30rem] truncate text-xs text-slate-400" title={rootMediaFolder}>
            Root media: {rootMediaFolder}
          </p>
        ) : null}
        {settingsMenu ? <div className="shrink-0">{settingsMenu}</div> : null}
      </div>
    </header>
  );
}
