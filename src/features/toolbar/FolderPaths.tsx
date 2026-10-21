interface FolderPathsProps {
  selectedFolder: string | null;
  destinationRoot: string | null;
  destinationError: string | null;
  renamesInPlace: boolean;
}

function PathValue({ path }: { path: string }) {
  return (
    <dd className="truncate" title={path}>
      {path}
    </dd>
  );
}

// Shows where files come from and where they go, so the person sees the move before starting it.
export function FolderPaths({
  selectedFolder,
  destinationRoot,
  destinationError,
  renamesInPlace
}: FolderPathsProps) {
  if (!selectedFolder) {
    return <p className="text-sm text-slate-400">No folder selected</p>;
  }

  return (
    <dl className="grid min-w-0 grid-cols-[auto_minmax(0,1fr)] gap-x-2 text-sm text-slate-200">
      <dt className="text-slate-400">From</dt>
      <PathValue path={selectedFolder} />
      <dt className="text-slate-400">To</dt>
      {destinationError ? (
        <dd className="truncate text-amber-300" title={destinationError}>
          Can't use the destination: {destinationError}
        </dd>
      ) : renamesInPlace ? (
        <dd className="truncate text-sky-300">Same folders, new names only</dd>
      ) : destinationRoot ? (
        <PathValue path={destinationRoot} />
      ) : (
        <dd className="text-slate-400">Checking...</dd>
      )}
    </dl>
  );
}
