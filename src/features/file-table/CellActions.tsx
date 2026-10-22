import { useCallback, useState, type MouseEvent, type ReactNode } from "react";
import { ContextMenu } from "../../components/design-system/ContextMenu";
import { openFile, revealFileInExplorer } from "../../lib/api";
import type { FileRowState } from "./fileRowState";

interface CellActionsProps {
  state: FileRowState;
  columnId: string;
  onOpenMap: (state: FileRowState) => void;
  children: ReactNode;
}

function filePathFor({ record, movePlan, moveResult }: FileRowState, columnId: string) {
  if (columnId === "source") {
    return record.source_path;
  }
  if (columnId === "destination") {
    return moveResult?.attempted_destination_path ?? movePlan?.destination_path;
  }
  return null;
}

function run(action: (path: string) => Promise<void>, path: string) {
  void action(path).catch((error: unknown) => {
    console.error(`Failed to open ${path}`, error);
  });
}

// Path cells open the file and offer its location on right click. A place cell with GPS opens the map.
export function CellActions({ state, columnId, onOpenMap, children }: CellActionsProps) {
  const [menuAt, setMenuAt] = useState<{ x: number; y: number } | null>(null);
  const closeMenu = useCallback(() => setMenuAt(null), []);
  const path = filePathFor(state, columnId);
  const hasMap = columnId === "place" && state.metadata?.gps != null;
  const onDoubleClick = path ? () => run(openFile, path) : hasMap ? () => onOpenMap(state) : null;
  const onContextMenu = (event: MouseEvent) => {
    event.preventDefault();
    setMenuAt({ x: event.clientX, y: event.clientY });
  };

  return (
    <div
      className={onDoubleClick ? "cursor-pointer" : undefined}
      onDoubleClick={onDoubleClick ?? undefined}
      onContextMenu={path ? onContextMenu : undefined}
    >
      {children}
      {path && menuAt ? (
        <ContextMenu
          x={menuAt.x}
          y={menuAt.y}
          onClose={closeMenu}
          items={[
            { label: "Open", onSelect: () => run(openFile, path) },
            { label: "Open file location", onSelect: () => run(revealFileInExplorer, path) }
          ]}
        />
      ) : null}
    </div>
  );
}
