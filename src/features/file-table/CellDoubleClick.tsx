import type { ReactNode } from "react";
import { revealFileInExplorer } from "../../lib/api";
import type { FileRowState } from "./fileRowState";

interface CellDoubleClickProps {
  state: FileRowState;
  columnId: string;
  onOpenMap: (state: FileRowState) => void;
  children: ReactNode;
}

function revealPathFor({ record, movePlan, moveResult }: FileRowState, columnId: string) {
  if (columnId === "source") {
    return record.source_path;
  }
  if (columnId === "destination") {
    return moveResult?.attempted_destination_path ?? movePlan?.destination_path;
  }
  return null;
}

function reveal(path: string) {
  void revealFileInExplorer(path).catch((error: unknown) => {
    console.error("Failed to reveal file in explorer", error);
  });
}

// Path cells reveal the file. A place cell with GPS opens the map.
export function CellDoubleClick({ state, columnId, onOpenMap, children }: CellDoubleClickProps) {
  const revealPath = revealPathFor(state, columnId);
  const hasMap = columnId === "place" && state.metadata?.gps != null;
  const action = revealPath ? () => reveal(revealPath) : hasMap ? () => onOpenMap(state) : null;

  return (
    <div className={action ? "cursor-pointer" : undefined} onDoubleClick={action ?? undefined}>
      {children}
    </div>
  );
}
