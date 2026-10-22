import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { HistoryMenu } from "./HistoryMenu";
import type { HistoryIndexEntry } from "../../lib/types";

const RUNS: HistoryIndexEntry[] = [
  {
    run_id: "run-a",
    folder_label: "TODO Camera",
    selected_folder: "C:\\TODO\\Camera",
    destination_root_path: null,
    started_at: "2026-05-24T00:00:00Z",
    completed_at: "2026-05-24T00:05:00Z",
    run_status: "completed",
    files_migrated: 0,
    undo_available: true,
    redo_available: false,
    history_file_path: "C:\\history\\run-a.jsonl"
  },
  {
    run_id: "run-b",
    folder_label: "TODO Drone",
    selected_folder: "C:\\TODO\\Drone",
    destination_root_path: null,
    started_at: "2026-05-23T00:00:00Z",
    completed_at: "2026-05-23T00:10:00Z",
    run_status: "undone",
    files_migrated: 0,
    undo_available: false,
    redo_available: true,
    history_file_path: "C:\\history\\run-b.jsonl"
  }
];

afterEach(cleanup);

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
        error={null}
        onSelectRun={onSelectHistoryRun}
        onUndoRun={onUndoProcessing}
        onRedoRun={onRedoProcessing}
        onDeleteRun={vi.fn()}
        onOpenLog={vi.fn()}
      />
    );

    expect(screen.getByText("History")).toBeInTheDocument();

    fireEvent.click(screen.getByText("History"));

    expect(screen.getByRole("table", { name: "Recent runs" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "C:\\TODO\\Camera" })).toHaveAttribute(
      "aria-current",
      "true"
    );

    fireEvent.click(screen.getByRole("button", { name: "Undo TODO Camera" }));
    expect(onUndoProcessing).toHaveBeenCalledWith("run-a");

    expect(screen.getByRole("button", { name: "Delete TODO Camera" })).toHaveTextContent("Delete");
    fireEvent.click(screen.getByRole("button", { name: "Redo TODO Drone" }));
    expect(onRedoProcessing).toHaveBeenCalledWith("run-b");

    fireEvent.click(screen.getByRole("button", { name: "C:\\TODO\\Drone" }));
    expect(onSelectHistoryRun).toHaveBeenCalledWith("run-b");
  });

  it("shows a history error even when no run is selected", () => {
    render(
      <HistoryMenu
        runs={[]}
        selectedRunId={null}
        loading={false}
        details={null}
        detailsLoading={false}
        error="Failed to open the history log: missing"
        onSelectRun={vi.fn()}
        onUndoRun={vi.fn()}
        onRedoRun={vi.fn()}
        onDeleteRun={vi.fn()}
        onOpenLog={vi.fn()}
      />
    );

    fireEvent.click(screen.getByText("History"));

    expect(screen.getByRole("alert")).toHaveTextContent("Failed to open the history log: missing");
  });
});
