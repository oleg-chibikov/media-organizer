import { act, renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { chooseFolder, startScan } from "../../lib/api";
import { useScanSession } from "./useScanSession";

vi.mock("../../lib/api", () => ({
  chooseFolder: vi.fn(),
  startScan: vi.fn(async () => undefined),
  cancelScan: vi.fn()
}));

async function choose(result: { current: ReturnType<typeof useScanSession> }, folder: string) {
  vi.mocked(chooseFolder).mockResolvedValueOnce(folder);
  await act(() => result.current.chooseFolderAndScan());
}

describe("useScanSession", () => {
  it("keeps the scanned folder when the new one is inside Processed", async () => {
    const reportError = vi.fn();
    const { result } = renderHook(() => useScanSession(reportError));

    await choose(result, "C:\\in");
    await choose(result, "C:\\in\\Processed");

    expect(result.current.selectedFolder).toBe("C:\\in");
    expect(startScan).toHaveBeenCalledTimes(1);
    expect(reportError).toHaveBeenLastCalledWith("Selected folder is inside a Processed path.");
  });

  it("goes back to the previous folder when the scan can't start", async () => {
    const { result } = renderHook(() => useScanSession(vi.fn()));

    await choose(result, "C:\\in");
    vi.mocked(startScan).mockRejectedValueOnce(new Error("busy"));
    await choose(result, "C:\\other");

    expect(result.current.selectedFolder).toBe("C:\\in");
  });
});
