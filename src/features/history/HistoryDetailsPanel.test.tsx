import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { HistoryDetailsPanel } from "./HistoryDetailsPanel";
import type { HistoryRunDetails } from "../../lib/types";

const DETAILS: HistoryRunDetails = {
  run: {
    run_id: "run-1",
    history_version: 1,
    app_version: "0.1.0",
    selected_folder: "C:\\TODO\\Camera",
    folder_label: "TODO Camera",
    started_at: "2026-05-24T00:00:00Z",
    completed_at: "2026-05-24T00:05:00Z",
    run_status: "completed",
    history_file_path: "C:\\history\\run-1.jsonl"
  },
  operations: [
    {
      operation_id: "op-process",
      run_id: "run-1",
      file_id: "f1",
      operation_kind: "process",
      source_path: "C:\\TODO\\Camera\\IMG_1.JPG",
      destination_path: "C:\\TODO\\Camera\\Processed\\2026\\05\\24\\IMG_1.JPG",
      expected_source_size: 100,
      expected_destination_size: 100,
      metadata_date: "2026-05-24T00:00:00Z",
      original_name: "IMG_1",
      move_strategy: "rename",
      status: "succeeded",
      error: null,
      timestamp: "2026-05-24T00:01:00Z"
    },
    {
      operation_id: "op-undo",
      run_id: "run-1",
      file_id: "f1",
      operation_kind: "undo",
      source_path: "C:\\TODO\\Camera\\Processed\\2026\\05\\24\\IMG_1.JPG",
      destination_path: "C:\\TODO\\Camera\\IMG_1.JPG",
      expected_source_size: 100,
      expected_destination_size: 100,
      metadata_date: null,
      original_name: "IMG_1",
      move_strategy: "rename",
      status: "blocked",
      error: "Destination already exists",
      timestamp: "2026-05-24T00:02:00Z"
    },
    {
      operation_id: "op-redo",
      run_id: "run-1",
      file_id: "f1",
      operation_kind: "redo",
      source_path: "C:\\TODO\\Camera\\IMG_1.JPG",
      destination_path: "C:\\TODO\\Camera\\Processed\\2026\\05\\24\\IMG_1.JPG",
      expected_source_size: 100,
      expected_destination_size: 100,
      metadata_date: null,
      original_name: "IMG_1",
      move_strategy: "rename",
      status: "failed",
      error: "File missing",
      timestamp: "2026-05-24T00:03:00Z"
    }
  ],
  unsafe_for_undo_redo: false,
  recovery_error: null
};

describe("HistoryDetailsPanel", () => {
  it("renders source/destination/status and exposes errors via status tooltip", () => {
    render(<HistoryDetailsPanel details={DETAILS} loading={false} error={null} />);

    expect(screen.getByText("History Details")).toBeInTheDocument();
    expect(screen.getByText("TODO Camera")).toBeInTheDocument();
    expect(screen.queryByText("Kind")).not.toBeInTheDocument();
    expect(screen.queryByText("Time")).not.toBeInTheDocument();
    expect(screen.queryByText("File")).not.toBeInTheDocument();
    expect(screen.queryByText("Size")).not.toBeInTheDocument();
    expect(screen.queryByText("Error")).not.toBeInTheDocument();
    expect(screen.getByText("Blocked")).toBeInTheDocument();
    expect(screen.getByText("Failed")).toBeInTheDocument();
    expect(screen.getByTitle("Blocked: Destination already exists")).toBeInTheDocument();
    expect(screen.getByTitle("Failed: File missing")).toBeInTheDocument();
  });
});
