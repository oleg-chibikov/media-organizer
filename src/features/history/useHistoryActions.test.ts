import { act, renderHook, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { deleteHistoryRun, revealFileInExplorer, undoHistoryRun } from "../../lib/api";
import { useHistoryActions } from "./useHistoryActions";

vi.mock("../../lib/api", () => ({
  deleteHistoryRun: vi.fn(async () => undefined),
  getHistoryRun: vi.fn(async () => null),
  redoHistoryRun: vi.fn(async () => undefined),
  revealFileInExplorer: vi.fn(async () => Promise.reject(new Error("missing"))),
  undoHistoryRun: vi.fn(async () => undefined)
}));

function renderActions() {
  const options = {
    activeActionRef: { current: null },
    reloadRuns: vi.fn(async () => undefined),
    dropRun: vi.fn(),
    setDetails: vi.fn(),
    setHistoryError: vi.fn(),
    reportError: vi.fn()
  };
  return { ...renderHook(() => useHistoryActions(options)), options };
}

afterEach(() => vi.restoreAllMocks());

describe("useHistoryActions", () => {
  it("calls the backend only after the person confirms", async () => {
    vi.spyOn(window, "confirm").mockReturnValueOnce(false).mockReturnValue(true);
    const { result } = renderActions();

    act(() => result.current.undoRun("run-1"));
    act(() => result.current.deleteRun("run-1"));

    await waitFor(() => expect(deleteHistoryRun).toHaveBeenCalledWith("run-1"));
    expect(undoHistoryRun).not.toHaveBeenCalled();
  });

  it("drops a deleted run before the backend answers", () => {
    vi.spyOn(window, "confirm").mockReturnValue(true);
    vi.mocked(deleteHistoryRun).mockReturnValueOnce(new Promise(() => undefined));
    const { result, options } = renderActions();

    act(() => result.current.deleteRun("run-1"));

    expect(options.dropRun).toHaveBeenCalledWith("run-1");
    expect(options.setDetails).toHaveBeenCalled();
  });

  it("brings a run back when the delete fails", async () => {
    vi.spyOn(window, "confirm").mockReturnValue(true);
    vi.mocked(deleteHistoryRun).mockRejectedValueOnce(new Error("locked"));
    const { result, options } = renderActions();

    act(() => result.current.deleteRun("run-1"));

    await waitFor(() => expect(options.reloadRuns).toHaveBeenCalled());
    expect(options.reportError).toHaveBeenCalledWith(
      "Failed to delete history entry: Error: locked"
    );
  });

  it("reports a log that can't be opened", async () => {
    const { result, options } = renderActions();

    act(() => result.current.openLog("C:\\history\\run-1.jsonl"));

    expect(revealFileInExplorer).toHaveBeenCalledWith("C:\\history\\run-1.jsonl");
    await waitFor(() =>
      expect(options.setHistoryError).toHaveBeenCalledWith(
        "Failed to open the history log: Error: missing"
      )
    );
  });
});
