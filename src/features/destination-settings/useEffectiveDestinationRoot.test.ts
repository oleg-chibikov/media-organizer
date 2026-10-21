import { renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { getEffectiveDestinationRoot } from "../../lib/api";
import type { DestinationSettings, EffectiveDestinationRoot } from "../../lib/types";
import { useEffectiveDestinationRoot } from "./useEffectiveDestinationRoot";

vi.mock("../../lib/api", () => ({ getEffectiveDestinationRoot: vi.fn() }));

const SETTINGS = { root_media_folder: null, force_processed_folder: false } as DestinationSettings;
const ROOT = { root_path: "C:\\in\\Processed" } as EffectiveDestinationRoot;

describe("useEffectiveDestinationRoot", () => {
  it("asks the backend again when the settings change", async () => {
    vi.mocked(getEffectiveDestinationRoot).mockResolvedValue(ROOT);
    const { result, rerender } = renderHook(
      ({ settings }) => useEffectiveDestinationRoot("C:\\in", settings),
      { initialProps: { settings: SETTINGS } }
    );

    await waitFor(() => expect(result.current.rootPath).toBe("C:\\in\\Processed"));

    vi.mocked(getEffectiveDestinationRoot).mockRejectedValue("Folder is offline");
    rerender({ settings: { ...SETTINGS, root_media_folder: "D:\\Media" } });

    expect(result.current.rootPath).toBeNull();
    await waitFor(() => expect(result.current.error).toBe("Folder is offline"));
    expect(getEffectiveDestinationRoot).toHaveBeenCalledTimes(2);
  });

  it("stays unknown without a selected folder", () => {
    const { result } = renderHook(() => useEffectiveDestinationRoot(null, SETTINGS));

    expect(result.current).toEqual({ rootPath: null, error: null });
  });
});
