import { act, renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { cancelScan, chooseFolder, startScan } from "../../lib/api";
import { RECORD } from "../../test/pipelineFixtures";
import { useScanSession } from "./useScanSession";

vi.mock("../../lib/api", () => ({
  chooseFolder: vi.fn(),
  startScan: vi.fn(async () => undefined),
  cancelScan: vi.fn(async () => undefined)
}));

type Session = { current: ReturnType<typeof useScanSession> };

function renderSession(reportError = vi.fn()) {
  const onNewScan = vi.fn();
  return { ...renderHook(() => useScanSession({ reportError, onNewScan })), onNewScan };
}

async function choose(result: Session, folder: string) {
  vi.mocked(chooseFolder).mockResolvedValueOnce(folder);
  await act(() => result.current.chooseFolderAndScan());
}

function scanOneFile(result: Session, root: string) {
  act(() => {
    result.current.events.onStarted({ root });
    result.current.events.onFiles({ records: [{ ...RECORD, source_path: `${root}\\a.jpg` }] });
    result.current.events.onFinished({ discovered: 1, skipped: 0, errors: 0, cancelled: false });
  });
}

describe("useScanSession", () => {
  it("keeps the scanned folder when the new one is inside Processed", async () => {
    const reportError = vi.fn();
    const { result } = renderSession(reportError);

    await choose(result, "C:\\in");
    await choose(result, "C:\\in\\Processed");

    expect(result.current.selectedFolder).toBe("C:\\in");
    expect(startScan).toHaveBeenCalledTimes(1);
    expect(reportError).toHaveBeenLastCalledWith("Selected folder is inside a Processed path.");
  });

  it("clears the old folder's files as soon as a new folder is picked", async () => {
    const { result, onNewScan } = renderSession();
    await choose(result, "C:\\in");
    scanOneFile(result, "C:\\in");
    onNewScan.mockClear();

    vi.mocked(startScan).mockReturnValueOnce(new Promise(() => undefined));
    vi.mocked(chooseFolder).mockResolvedValueOnce("C:\\other");
    await act(async () => void result.current.chooseFolderAndScan());

    expect(result.current.rows).toEqual([]);
    expect(result.current.progress.discovered).toBe(0);
    expect(result.current.isScanning).toBe(true);
    expect(result.current.selectedFolder).toBe("C:\\other");
    expect(onNewScan).toHaveBeenCalledOnce();
  });

  it("drops late events from the replaced scan until the new one starts", async () => {
    const { result } = renderSession();
    await choose(result, "C:\\in");
    await choose(result, "C:\\other");

    scanOneFile(result, "C:\\in");
    expect(result.current.rows).toEqual([]);
    expect(result.current.isScanning).toBe(true);

    scanOneFile(result, "C:\\other");
    expect(result.current.rows).toHaveLength(1);
    expect(result.current.progress.completed).toBe(true);
  });

  it("clears the folder when the scan can't start", async () => {
    const { result } = renderSession();

    await choose(result, "C:\\in");
    vi.mocked(startScan).mockRejectedValueOnce(new Error("busy"));
    await choose(result, "C:\\other");

    expect(result.current.selectedFolder).toBeNull();
    expect(result.current.isScanning).toBe(false);
  });

  it("shows the scan as stopped before the backend answers", async () => {
    const { result } = renderSession();
    await choose(result, "C:\\in");
    act(() => result.current.events.onStarted({ root: "C:\\in" }));

    vi.mocked(cancelScan).mockReturnValueOnce(new Promise(() => undefined));
    act(() => void result.current.stopScan());

    expect(result.current.isScanning).toBe(false);
    expect(result.current.progress.cancelled).toBe(true);
  });

  it("counts found and skipped files from one batch", async () => {
    const { result } = renderSession();
    await choose(result, "C:\\in");
    const skipped = { ...RECORD, id: "b", scan_status: "skipped_unsupported" as const };

    act(() => {
      result.current.events.onStarted({ root: "C:\\in" });
      result.current.events.onFiles({ records: [RECORD, skipped, RECORD] });
    });

    expect(result.current.progress).toMatchObject({ discovered: 1, skipped: 1 });
  });
});
