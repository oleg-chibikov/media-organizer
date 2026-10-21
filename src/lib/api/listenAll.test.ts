import { afterEach, describe, expect, it, vi } from "vitest";
import { listenAll } from "./listenAll";

const registered = vi.hoisted(() => new Map<string, (event: { payload: unknown }) => void>());

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async (name: string, callback: (event: { payload: unknown }) => void) => {
    registered.set(name, callback);
    return () => registered.delete(name);
  })
}));

type TestWindow = Window & { __TAURI_INTERNALS__?: unknown };

afterEach(() => {
  registered.clear();
  delete (window as TestWindow).__TAURI_INTERNALS__;
});

describe("listenAll", () => {
  it("subscribes provided handlers to their event names and forwards payloads", async () => {
    (window as TestWindow).__TAURI_INTERNALS__ = {};
    const onPing = vi.fn();

    const unlisteners = await listenAll(
      { onPing: "test:ping", onPong: "test:pong" },
      { onPing, onPong: undefined }
    );
    registered.get("test:ping")?.({ payload: "hello" });

    expect([...registered.keys()]).toEqual(["test:ping"]);
    expect(onPing).toHaveBeenCalledWith("hello");
    expect(unlisteners).toHaveLength(1);
  });

  it("refuses to listen outside the Tauri runtime", async () => {
    await expect(listenAll({ onPing: "test:ping" }, { onPing: vi.fn() })).rejects.toThrow(
      "Tauri runtime is unavailable"
    );
  });
});
