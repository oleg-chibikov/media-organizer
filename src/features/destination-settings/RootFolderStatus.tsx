export function RootFolderStatus({ rootPath }: { rootPath: string | null }) {
  return (
    <div className="mt-3 rounded border border-slate-800 bg-slate-950 p-2">
      <p className="text-[11px] uppercase tracking-wide text-slate-400">Library folder</p>
      {rootPath ? (
        <p className="mt-1 break-all font-mono text-xs text-slate-200">{rootPath}</p>
      ) : (
        <p className="mt-1 text-xs text-slate-400">
          Not set. Files go to a Processed folder inside the folder you scan.
        </p>
      )}
    </div>
  );
}
