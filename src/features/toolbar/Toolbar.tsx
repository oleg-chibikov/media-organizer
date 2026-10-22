import type { ReactNode } from "react";
import { Button } from "../../components/design-system/Button";
import { ErrorBanner } from "./ErrorBanner";
import { FolderPaths } from "./FolderPaths";

interface ToolbarProps {
  selectedFolder: string | null;
  destinationRoot: string | null;
  destinationError: string | null;
  renamesInPlace?: boolean;
  folderError: string | null;
  isScanning: boolean;
  movesRunning: boolean;
  historyMenu?: ReactNode;
  placeToggle?: ReactNode;
  renameToggle?: ReactNode;
  settingsMenu?: ReactNode;
  onChooseFolder: () => void;
  onCancelScan: () => void;
  onDismissError: () => void;
}

export function Toolbar({
  selectedFolder,
  destinationRoot,
  destinationError,
  renamesInPlace = false,
  folderError,
  isScanning,
  movesRunning,
  historyMenu,
  placeToggle,
  renameToggle,
  settingsMenu,
  onChooseFolder,
  onCancelScan,
  onDismissError
}: ToolbarProps) {
  return (
    <header className="border-b border-slate-800 bg-slate-900">
      <div className="flex items-center gap-2 px-3 py-2">
        {/* A new scan resets the move state while the backend is still moving files. */}
        <Button
          variant="primary"
          className="shrink-0 px-3 py-1.5 text-sm"
          disabled={movesRunning}
          title={movesRunning ? "Wait for the files to finish moving" : undefined}
          onClick={onChooseFolder}
        >
          Choose Folder
        </Button>
        {isScanning ? (
          <Button
            variant="secondary"
            className="shrink-0 bg-rose-600 px-3 py-1.5 text-sm text-white hover:bg-rose-500"
            onClick={onCancelScan}
          >
            Stop Scan
          </Button>
        ) : null}
        <div className="shrink-0">{historyMenu}</div>

        <div className="ml-2 min-w-0 flex-1">
          <FolderPaths
            selectedFolder={selectedFolder}
            destinationRoot={destinationRoot}
            destinationError={destinationError}
            renamesInPlace={renamesInPlace}
          />
        </div>

        {placeToggle ? <div className="shrink-0">{placeToggle}</div> : null}
        {renameToggle ? <div className="shrink-0">{renameToggle}</div> : null}
        {settingsMenu ? <div className="shrink-0">{settingsMenu}</div> : null}
      </div>
      {folderError ? <ErrorBanner message={folderError} onDismiss={onDismissError} /> : null}
    </header>
  );
}
