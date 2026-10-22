import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import App from "./App";
import type { MoveListeners, PlanListeners, ScanListeners } from "./lib/api";
import { METADATA, PLAN, RECORD } from "./test/pipelineFixtures";

const listeners = vi.hoisted(() => ({
  scan: null as ScanListeners | null,
  plan: null as PlanListeners | null,
  move: null as MoveListeners | null
}));

vi.mock("./lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("./lib/api")>()),
  healthCheck: vi.fn(async () => "ok"),
  getProcessingConfig: vi.fn(async () => ({
    metadata_concurrency: 1,
    metadata_queue_capacity: 256,
    metadata_timeout_ms: 1000
  })),
  getDestinationSettings: vi.fn(async () => ({
    version: 1,
    root_media_folder: null,
    force_processed_folder: false,
    rename_in_place: false,
    home: null,
    updated_at: "2026-05-24T00:00:00Z"
  })),
  getEffectiveDestinationRoot: vi.fn(async () => ({ root_path: "C:\\in\\Processed" })),
  listHistoryRuns: vi.fn(async () => []),
  chooseFolder: vi.fn(async () => "C:\\in"),
  startScan: vi.fn(async () => undefined),
  readMetadataForFiles: vi.fn(async () => [METADATA]),
  createMovePlan: vi.fn(async () => undefined),
  getMoveConfirmationSummary: vi.fn(async () => ({
    ready_operations: 1,
    skipped_operations: 0,
    fallback_warnings: 0,
    suffix_applied: 0
  })),
  executeMovePlan: vi.fn(async () => undefined),
  pauseMoveRun: vi.fn(async () => undefined),
  removeEmptySourceFolders: vi.fn(async () => false),
  revealFileInExplorer: vi.fn(async () => undefined),
  attachScanListeners: vi.fn(async (handlers: ScanListeners) => ((listeners.scan = handlers), [])),
  attachPlanListeners: vi.fn(async (handlers: PlanListeners) => ((listeners.plan = handlers), [])),
  attachMoveListeners: vi.fn(async (handlers: MoveListeners) => ((listeners.move = handlers), [])),
  attachHistoryListeners: vi.fn(async () => [])
}));

const api = await import("./lib/api");

afterEach(cleanup);

describe("App", () => {
  it("scans, plans and executes moves through the backend events", async () => {
    render(<App />);
    await waitFor(() => expect(api.healthCheck).toHaveBeenCalled());

    expect(screen.getByText("Organize photos and videos into folders by date")).toBeInTheDocument();
    fireEvent.click(screen.getAllByRole("button", { name: "Choose Folder" })[0]);
    await waitFor(() => expect(api.startScan).toHaveBeenCalledWith("C:\\in"));

    act(() => {
      listeners.scan?.onStarted({ root: "C:\\in" });
      listeners.scan?.onFile({ record: RECORD });
      listeners.scan?.onFinished({ discovered: 1, skipped: 0, errors: 0, cancelled: false });
    });
    await waitFor(() =>
      expect(api.createMovePlan).toHaveBeenCalledWith("C:\\in", [RECORD], [METADATA], [])
    );

    act(() => {
      listeners.plan?.onFile({ plan: PLAN });
      listeners.plan?.onFinished({ planned: 1, errors: 0 });
    });
    expect(screen.getByRole("status")).toHaveTextContent("Ready to move");
    expect(screen.getByText("Ready: 1")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Move Files..." }));
    fireEvent.click(await screen.findByRole("button", { name: "Move 1 file" }));
    await waitFor(() =>
      expect(api.executeMovePlan).toHaveBeenCalledWith("C:\\in", [RECORD], [METADATA], [PLAN])
    );

    const result = {
      file_id: "f1",
      attempted_source_path: RECORD.source_path,
      attempted_destination_path: PLAN.destination_path,
      move_strategy: "rename" as const,
      result_status: "moved" as const,
      error: null
    };
    act(() => {
      listeners.move?.onFile({ result });
      listeners.move?.onFinished({ moved: 1, copied: 0, skipped: 0, failed: 0, results: [result] });
    });
    expect(screen.getByRole("status")).toHaveTextContent("Done");
    expect(screen.getByText("Moved: 1")).toBeInTheDocument();
    expect(screen.getByText(/Moved 1 file\./)).toBeInTheDocument();
    expect(api.removeEmptySourceFolders).toHaveBeenCalledWith("C:\\in");
    fireEvent.click(screen.getByRole("button", { name: "Open Destination" }));
    expect(api.revealFileInExplorer).toHaveBeenCalledWith("C:\\in\\Processed");
  });
});
