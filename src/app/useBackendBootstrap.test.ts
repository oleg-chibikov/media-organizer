import { renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { DestinationSettings } from "../lib/types";
import { useBackendBootstrap } from "./useBackendBootstrap";

const SETTINGS = { root_media_folder: null } as DestinationSettings;

vi.mock("../lib/api", () => ({
  healthCheck: vi.fn(async () => Promise.reject(new Error("no health"))),
  getDestinationSettings: vi.fn(async () => SETTINGS)
}));

describe("useBackendBootstrap", () => {
  it("loads settings even when health fails", async () => {
    const callbacks = {
      onSettingsLoaded: vi.fn(),
      onSettingsFailed: vi.fn()
    };
    const { result } = renderHook(() => useBackendBootstrap(callbacks));

    await waitFor(() => expect(callbacks.onSettingsLoaded).toHaveBeenCalledWith(SETTINGS));
    await waitFor(() => expect(result.current.health).toBe("error: Error: no health"));
    expect(callbacks.onSettingsFailed).not.toHaveBeenCalled();
  });
});
