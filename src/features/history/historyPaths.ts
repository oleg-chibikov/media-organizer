interface HistoryRoots {
  selected_folder: string;
  destination_root_path?: string | null;
}

// Runs recorded before custom roots existed always moved files into `<folder>\Processed`.
export function historyDestinationRoot(run: HistoryRoots): string {
  return run.destination_root_path ?? `${run.selected_folder}\\Processed`;
}

export function toRelativeHistoryPath(path: string, root: string): string {
  const normalizedPath = path.replace(/\//g, "\\").toLowerCase();
  const normalizedRoot = root.replace(/\//g, "\\").toLowerCase();
  if (normalizedPath === normalizedRoot) {
    return ".";
  }
  const rootedPrefix = normalizedRoot.endsWith("\\") ? normalizedRoot : `${normalizedRoot}\\`;
  if (!normalizedPath.startsWith(rootedPrefix)) {
    return path;
  }
  return path.slice(rootedPrefix.length);
}
