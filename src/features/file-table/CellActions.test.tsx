import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { openFile, revealFileInExplorer } from "../../lib/api";
import type { MetadataResult } from "../../lib/types";
import { CellActions } from "./CellActions";
import type { FileRowState } from "./fileRowState";

vi.mock("../../lib/api", () => ({
  openFile: vi.fn(async () => undefined),
  revealFileInExplorer: vi.fn(async () => undefined)
}));

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

function rowState(gps: MetadataResult["gps"] = null): FileRowState {
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

function renderCell(columnId: string, state = rowState(), onOpenMap = vi.fn()) {
  render(
    <CellActions state={state} columnId={columnId} onOpenMap={onOpenMap}>
      cell
    </CellActions>
  );
  return onOpenMap;
}

describe("CellActions", () => {
  it("opens the map from a place cell with GPS", () => {
    const state = rowState({ latitude: -33.86, longitude: 151.21 });
    const onOpenMap = renderCell("place", state);
    fireEvent.doubleClick(screen.getByText("cell"));
    expect(onOpenMap).toHaveBeenCalledWith(state);
  });

  it("ignores a place cell without GPS", () => {
    const onOpenMap = renderCell("place");
    fireEvent.doubleClick(screen.getByText("cell"));
    expect(onOpenMap).not.toHaveBeenCalled();
  });

  it("opens the source file on double click", () => {
    const onOpenMap = renderCell("source", rowState({ latitude: 1, longitude: 2 }));
    fireEvent.doubleClick(screen.getByText("cell"));
    expect(openFile).toHaveBeenCalledWith("/in/a.jpg");
    expect(revealFileInExplorer).not.toHaveBeenCalled();
    expect(onOpenMap).not.toHaveBeenCalled();
  });

  it("offers the file location on right click", () => {
    renderCell("source");
    fireEvent.contextMenu(screen.getByText("cell"));
    expect(screen.getByRole("menuitem", { name: "Open" })).toHaveFocus();

    fireEvent.click(screen.getByRole("menuitem", { name: "Open file location" }));
    expect(revealFileInExplorer).toHaveBeenCalledWith("/in/a.jpg");
    expect(screen.queryByRole("menu")).not.toBeInTheDocument();
  });

  it("closes the menu on Escape", () => {
    renderCell("source");
    fireEvent.contextMenu(screen.getByText("cell"));
    fireEvent.keyDown(screen.getByRole("menu"), { key: "Escape" });
    expect(screen.queryByRole("menu")).not.toBeInTheDocument();
  });

  it("keeps the browser menu on cells without a path", () => {
    renderCell("place");
    fireEvent.contextMenu(screen.getByText("cell"));
    expect(screen.queryByRole("menu")).not.toBeInTheDocument();
  });
});
