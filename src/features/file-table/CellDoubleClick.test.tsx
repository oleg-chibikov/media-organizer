import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { MetadataResult } from "../../lib/types";
import { CellDoubleClick } from "./CellDoubleClick";
import type { FileRowState } from "./fileRowState";

const { revealFileInExplorer } = vi.hoisted(() => ({
  revealFileInExplorer: vi.fn(() => Promise.resolve())
}));
vi.mock("../../lib/api", () => ({ revealFileInExplorer }));

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

function rowState(gps: MetadataResult["gps"]): FileRowState {
  return {
    record: {
      id: "a",
      source_path: "/in/a.jpg",
      relative_path: "a.jpg",
      extension: "jpg",
      original_name: "a.jpg",
      kind: "photo",
      scan_status: "discovered"
    },
    metadata: { gps } as MetadataResult,
    movePlan: undefined,
    moveResult: undefined,
    planError: undefined
  };
}

function renderCell(columnId: string, state: FileRowState, onOpenMap = vi.fn()) {
  render(
    <CellDoubleClick state={state} columnId={columnId} onOpenMap={onOpenMap}>
      cell
    </CellDoubleClick>
  );
  fireEvent.doubleClick(screen.getByText("cell"));
  return onOpenMap;
}

describe("CellDoubleClick", () => {
  it("opens the map from a place cell with GPS", () => {
    const state = rowState({ latitude: -33.86, longitude: 151.21 });
    expect(renderCell("place", state)).toHaveBeenCalledWith(state);
  });

  it("ignores a place cell without GPS", () => {
    expect(renderCell("place", rowState(null))).not.toHaveBeenCalled();
  });

  it("reveals the source file from the source cell", () => {
    const onOpenMap = renderCell("source", rowState({ latitude: 1, longitude: 2 }));
    expect(revealFileInExplorer).toHaveBeenCalledWith("/in/a.jpg");
    expect(onOpenMap).not.toHaveBeenCalled();
  });
});
