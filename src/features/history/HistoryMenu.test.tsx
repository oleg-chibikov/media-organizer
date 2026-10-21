import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { HistoryMenu } from "./HistoryMenu";
import type { HistoryIndexEntry } from "../../lib/types";

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

describe("History Menu", () => {
  it("shows a single undo or redo action next to each run", () => {
    const onSelectHistoryRun = vi.fn();
    const onUndoProcessing = vi.fn();
    const onRedoProcessing = vi.fn();

    render(
      <HistoryMenu
        runs={RUNS}
        selectedRunId={"run-a"}
        loading={false}
        details={null}
        detailsLoading={false}
        detailsError={null}
        onSelectRun={onSelectHistoryRun}
        onUndoRun={onUndoProcessing}
        onRedoRun={onRedoProcessing}
        onDeleteRun={vi.fn()}
      />
    );

    expect(screen.getByText("History")).toBeInTheDocument();

    fireEvent.click(screen.getByText("History"));

    expect(screen.getByText("C:\\TODO\\Camera")).toBeInTheDocument();
    expect(screen.getByText("C:\\TODO\\Drone")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Undo TODO Camera" }));
    expect(onUndoProcessing).toHaveBeenCalledWith("run-a");

    fireEvent.click(screen.getByRole("button", { name: "Redo TODO Drone" }));
    expect(onRedoProcessing).toHaveBeenCalledWith("run-b");

    fireEvent.click(screen.getAllByRole("button", { name: "View run run-b" })[0]);
    expect(onSelectHistoryRun).toHaveBeenCalledWith("run-b");
  });
});
