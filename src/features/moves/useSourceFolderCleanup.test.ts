import { renderHook } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { removeEmptySourceFolders } from "../../lib/api";
import { useSourceFolderCleanup } from "./useSourceFolderCleanup";

vi.mock("../../lib/api", () => ({ removeEmptySourceFolders: vi.fn() }));

const FOLDER = "/media/TODO";

function renderCleanup(completed: boolean, selectedFolder: string | null = FOLDER) {
  const reportError = vi.fn();
  const hook = renderHook(
    (props: { completed: boolean; selectedFolder: string | null }) =>
      useSourceFolderCleanup(props.completed, props.selectedFolder, reportError),
    { initialProps: { completed, selectedFolder } }
  );
  return { ...hook, reportError };
}

afterEach(() => vi.resetAllMocks());

describe("useSourceFolderCleanup", () => {
  it("cleans the selected folder once when a run finishes", () => {
    vi.mocked(removeEmptySourceFolders).mockResolvedValue(false);
    const { rerender } = renderCleanup(false);
    expect(removeEmptySourceFolders).not.toHaveBeenCalled();

    rerender({ completed: true, selectedFolder: FOLDER });
    rerender({ completed: true, selectedFolder: FOLDER });

    expect(removeEmptySourceFolders).toHaveBeenCalledOnce();
    expect(removeEmptySourceFolders).toHaveBeenCalledWith(FOLDER);
  });

  it("leaves a newly picked folder alone while the old run still shows as finished", () => {
    const { rerender } = renderCleanup(true);

    rerender({ completed: true, selectedFolder: "/media/OTHER" });

    expect(removeEmptySourceFolders).not.toHaveBeenCalled();
  });

  it("reports a failed cleanup", async () => {
    vi.mocked(removeEmptySourceFolders).mockRejectedValue("denied");
    const { rerender, reportError } = renderCleanup(false);

    rerender({ completed: true, selectedFolder: FOLDER });

    await vi.waitFor(() =>
      expect(reportError).toHaveBeenCalledWith("Failed to remove empty source folders: denied")
    );
  });
});
