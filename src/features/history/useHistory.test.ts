import { act, renderHook, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { getHistoryRun, listHistoryRuns, type HistoryListeners } from "../../lib/api";
import type { HistoryIndexEntry, HistoryRunDetails, HistoryRunStatus } from "../../lib/types";
import { useHistory } from "./useHistory";

let handlers: HistoryListeners | null = null;

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  attachHistoryListeners: vi.fn(async (next: HistoryListeners) => {
    handlers = next;
    return [];
  }),
  listHistoryRuns: vi.fn(),
  getHistoryRun: vi.fn()
}));

const RUN = { run_id: "run-1" } as HistoryIndexEntry;

function detailsWithStatus(status: HistoryRunStatus) {
  return {
    run: { run_id: "run-1", run_status: status },
    operations: []
  } as unknown as HistoryRunDetails;
}

afterEach(() => {
  handlers = null;
  vi.clearAllMocks();
});

describe("useHistory", () => {
  it("reloads the selected run details when the backend updates that run", async () => {
    vi.mocked(listHistoryRuns).mockResolvedValue([RUN]);
    vi.mocked(getHistoryRun)
      .mockResolvedValueOnce(detailsWithStatus("running"))
      .mockResolvedValueOnce(detailsWithStatus("completed"));
    const { result } = renderHook(() => useHistory(vi.fn()));
    await waitFor(() => expect(handlers).not.toBeNull());

    act(() => handlers!.onIndexUpdated({ run_id: "run-1" }));
    await waitFor(() => expect(result.current.details?.run.run_status).toBe("running"));

    act(() => handlers!.onIndexUpdated({ run_id: "run-1" }));
    await waitFor(() => expect(result.current.details?.run.run_status).toBe("completed"));
  });

  it("keeps a deleted run unselected", async () => {
    vi.mocked(listHistoryRuns).mockResolvedValue([]);
    const { result } = renderHook(() => useHistory(vi.fn()));
    await waitFor(() => expect(handlers).not.toBeNull());

    act(() => handlers!.onIndexUpdated({ run_id: "deleted" }));

    await waitFor(() => expect(listHistoryRuns).toHaveBeenCalledTimes(2));
    expect(result.current.selectedRunId).toBeNull();
    expect(getHistoryRun).not.toHaveBeenCalled();
  });
});
