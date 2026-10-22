import { act, cleanup, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import App from "./App";
import type { MetadataListeners, MoveListeners, PlanListeners, ScanListeners } from "./lib/api";
import { METADATA, PLAN, RECORD } from "./test/pipelineFixtures";

const listeners = vi.hoisted(() => ({
  scan: null as ScanListeners | null,
  metadata: null as MetadataListeners | null,
  plan: null as PlanListeners | null,
  move: null as MoveListeners | null
}));

vi.mock("./lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("./lib/api")>()),
  healthCheck: vi.fn(async () => "ok"),
  getDestinationSettings: vi.fn(async () => ({
    version: 1,
    root_media_folder: null,
    force_processed_folder: false,
    rename_in_place: false,
    group_by_place: true,
    home: null,
    updated_at: "2026-05-24T00:00:00Z"
  })),
  getEffectiveDestinationRoot: vi.fn(async () => ({ root_path: "C:\\in\\Processed" })),
  listHistoryRuns: vi.fn(async () => []),
  chooseFolder: vi.fn(async () => "C:\\in"),
  startScan: vi.fn(async () => undefined),
  setMetadataPaused: vi.fn(async () => undefined),
  applyPlaceSuggestions: vi.fn(async () => undefined),
  detectPlaceSuggestions: vi.fn(async () => ({
    suggestions: [],
    file_places: {},
    status: { kind: "no_gps" }
  })),
  getMoveConfirmationSummary: vi.fn(async () => ({
    ready_operations: 1,
    skipped_operations: 0,
    fallback_warnings: 0,
    suffix_applied: 0
  })),
  startMoves: vi.fn(async () => undefined),
  revealFileInExplorer: vi.fn(async () => undefined),
  attachScanListeners: vi.fn(async (handlers: ScanListeners) => ((listeners.scan = handlers), [])),
  attachMetadataListeners: vi.fn(
    async (handlers: MetadataListeners) => ((listeners.metadata = handlers), [])
  ),
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
      listeners.scan?.onStarted({ root: "C:\\in", scan_id: 1 });
      listeners.scan?.onFiles({ records: [RECORD] });
      listeners.scan?.onFinished({ discovered: 1, skipped: 0, errors: 0, cancelled: false });
      listeners.metadata?.onResults({ scan_id: 1, results: [METADATA] });
      listeners.metadata?.onFinished({ scan_id: 1, cancelled: false });
    });
    // Plans wait for place detection, so no file shows a date folder first.
    expect(api.applyPlaceSuggestions).not.toHaveBeenCalled();
    await waitFor(() => expect(api.applyPlaceSuggestions).toHaveBeenCalledWith(1, []), {
      timeout: 3000
    });

    act(() => {
      listeners.plan?.onFile({ scan_id: 1, plan: PLAN });
      listeners.plan?.onSettled({ scan_id: 1 });
    });
    const statusBar = within(screen.getByRole("contentinfo"));
    expect(statusBar.getByRole("status")).toHaveTextContent("Ready to move");
    const placeToggle = screen.getByRole("checkbox", { name: "Group by place" });
    expect(placeToggle.closest("label")?.title).toContain("No GPS in these files.");
    expect(screen.getByText("Ready: 1")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Move Files..." }));
    fireEvent.click(await screen.findByRole("button", { name: "Move 1 file" }));
    await waitFor(() => expect(api.startMoves).toHaveBeenCalledWith(1));

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
      listeners.move?.onDone({ scan_id: 1, removed_selected_folder: false, error: null });
    });
    expect(statusBar.getByRole("status")).toHaveTextContent("Done");
    expect(screen.getByText("Moved: 1")).toBeInTheDocument();
    expect(screen.getByText(/Moved 1 file\./)).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Open Destination" }));
    expect(api.revealFileInExplorer).toHaveBeenCalledWith("C:\\in\\Processed");
  });
});
