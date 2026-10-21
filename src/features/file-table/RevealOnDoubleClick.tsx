import type { ReactNode } from "react";
import { revealFileInExplorer } from "../../lib/api";
import type { FileRowState } from "./fileRowState";

interface RevealOnDoubleClickProps {
  state: FileRowState;
  columnId: string;
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

export function RevealOnDoubleClick({ state, columnId, children }: RevealOnDoubleClickProps) {
  const isPathCell = columnId === "source" || columnId === "destination";
  const revealPath = revealPathFor(state, columnId);

  return (
    <div
      className={isPathCell ? "cursor-pointer" : undefined}
      onDoubleClick={() => {
        if (!revealPath) {
          return;
        }
        void revealFileInExplorer(revealPath).catch((error: unknown) => {
          console.error("Failed to reveal file in explorer", error);
        });
      }}
    >
      {children}
    </div>
  );
}
