interface ToolbarProps {
  selectedFolder: string | null;
  folderError: string | null;
  isScanning: boolean;
  onChooseFolder: () => void;
  onCancelScan: () => void;
}

export function Toolbar({
  selectedFolder,
  folderError,
  isScanning,
  onChooseFolder,
  onCancelScan
}: ToolbarProps) {
  return (
    <header className="flex flex-wrap items-center gap-2 border-b border-slate-800 bg-slate-900 px-3 py-2">
      <button
        className="rounded bg-sky-600 px-3 py-1.5 text-sm font-medium text-white hover:bg-sky-500 disabled:cursor-not-allowed disabled:bg-slate-700"
        onClick={onChooseFolder}
        type="button"
      >
        Choose Folder
      </button>
      <button
        className="rounded bg-rose-600 px-3 py-1.5 text-sm font-medium text-white hover:bg-rose-500 disabled:cursor-not-allowed disabled:bg-slate-700"
        disabled={!isScanning}
        onClick={onCancelScan}
        type="button"
      >
        Cancel
      </button>

      <div className="ml-2 min-w-0 flex-1 text-sm text-slate-300">
        {selectedFolder ? (
          <p className="truncate">{selectedFolder}</p>
        ) : (
          <p className="text-slate-500">No folder selected</p>
        )}
        {folderError ? <p className="text-rose-400">{folderError}</p> : null}
      </div>
    </header>
  );
}
