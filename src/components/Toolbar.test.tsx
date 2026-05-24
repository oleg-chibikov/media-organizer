import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { Toolbar } from "./Toolbar";
import type { HistoryIndexEntry } from "../lib/types";

const RUNS: HistoryIndexEntry[] = [
  {
    run_id: "run-a",
    folder_label: "TODO Camera",
    selected_folder: "C:\\TODO\\Camera",
    started_at: "2026-05-24T00:00:00Z",
    completed_at: "2026-05-24T00:05:00Z",
    run_status: "completed",
    undo_available: true,
    redo_available: false,
    history_file_path: "C:\\history\\run-a.jsonl"
  },
  {
    run_id: "run-b",
    folder_label: "TODO Drone",
    selected_folder: "C:\\TODO\\Drone",
    started_at: "2026-05-23T00:00:00Z",
    completed_at: "2026-05-23T00:10:00Z",
    run_status: "undone",
    undo_available: false,
    redo_available: true,
    history_file_path: "C:\\history\\run-b.jsonl"
  }
];

describe("Toolbar History Menu", () => {
  it("shows a single undo or redo action next to each run", () => {
    const onSelectHistoryRun = vi.fn();
    const onUndoProcessing = vi.fn();
    const onRedoProcessing = vi.fn();

    render(
      <Toolbar
        selectedFolder={null}
        folderError={null}
        isScanning={false}
        historyRuns={RUNS}
        selectedHistoryRunId={"run-a"}
        historyLoading={false}
        onChooseFolder={vi.fn()}
        onCancelScan={vi.fn()}
        onSelectHistoryRun={onSelectHistoryRun}
        onUndoProcessing={onUndoProcessing}
        onRedoProcessing={onRedoProcessing}
      />
    );

    expect(screen.getByText("History")).toBeInTheDocument();
    expect(screen.getByText("TODO Camera")).toBeInTheDocument();
    expect(screen.getByText("TODO Drone")).toBeInTheDocument();

    const undoButton = screen.getByRole("button", { name: "Undo TODO Camera" });
    const redoButton = screen.getByRole("button", { name: "Redo TODO Drone" });

    fireEvent.click(undoButton);
    expect(onUndoProcessing).toHaveBeenCalledWith("run-a");

    fireEvent.click(redoButton);
    expect(onRedoProcessing).toHaveBeenCalledWith("run-b");

    fireEvent.click(screen.getByRole("button", { name: "View TODO Drone" }));
    expect(onSelectHistoryRun).toHaveBeenCalledWith("run-b");
  });
});
