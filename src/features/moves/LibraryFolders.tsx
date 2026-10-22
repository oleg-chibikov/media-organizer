import type { LibraryFolderMove } from "../../lib/types";
import { fileCountLabel } from "./moveConfirmationPrompt";

interface LibraryFoldersProps {
  gather: boolean;
  folders: LibraryFolderMove[];
  destinationRoot: string | null;
  busy: boolean;
  onChange: (gather: boolean) => void;
}

// Folders from earlier runs with the same places on nearby days, which move in this run too.
export function LibraryFolders({
  gather,
  folders,
  destinationRoot,
  busy,
  onChange
}: LibraryFoldersProps) {
  if (gather && folders.length === 0) {
    return null;
  }
  const shown = (path: string) => withinRoot(path, destinationRoot);
  return (
    <section className="mt-3" aria-label="Earlier days">
      <label className="flex cursor-pointer items-start gap-2 has-disabled:cursor-wait">
        <input
          type="checkbox"
          className="mt-1"
          checked={gather}
          disabled={busy}
          onChange={(event) => onChange(event.target.checked)}
        />
        <span>Also move earlier days of these places into the same folder</span>
      </label>
      {gather ? (
        <ul className="mt-1 space-y-1 pl-6 text-slate-300">
          {folders.map((folder) => (
            <li key={folder.from}>
              {fileCountLabel(folder.files)}:{" "}
              <span className="break-all font-mono text-xs">{shown(folder.from)}</span> to{" "}
              <span className="break-all font-mono text-xs">{shown(folder.to)}</span>
            </li>
          ))}
        </ul>
      ) : null}
    </section>
  );
}

function withinRoot(path: string, root: string | null): string {
  if (!root || !path.startsWith(root)) {
    return path;
  }
  return path.slice(root.length).replace(/^[\\/]+/, "");
}
