import type { PreviewFolder } from "./placeFolderPreview";

export function FolderPreviewList({ folders }: { folders: PreviewFolder[] }) {
  return (
    <ul
      aria-label="Folder preview"
      className="mt-3 max-h-40 overflow-auto rounded bg-slate-950 p-2 font-mono text-xs text-slate-200"
    >
      {folders.map((folder) => (
        <li key={folder.path}>
          {folder.path} ({folder.count})
        </li>
      ))}
    </ul>
  );
}
