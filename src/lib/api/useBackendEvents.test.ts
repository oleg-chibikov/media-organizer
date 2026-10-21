import { cleanup, renderHook, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { useBackendEvents } from "./useBackendEvents";

interface Handlers {
  onPing: (payload: string) => void;
}

afterEach(cleanup);

describe("useBackendEvents", () => {
  it("forwards events to the latest handlers and unlistens on unmount", async () => {
    let attached: Handlers | null = null;
    const unlisten = vi.fn();
    const attach = vi.fn(async (handlers: Handlers) => {
      attached = handlers;
      return [unlisten];
    });
    const first = vi.fn();
    const second = vi.fn();

    const { rerender, unmount } = renderHook(
      ({ onPing }: Handlers) => useBackendEvents(attach, { onPing }, vi.fn()),
      { initialProps: { onPing: first } }
    );
    await waitFor(() => expect(attached).not.toBeNull());
    rerender({ onPing: second });
    attached!.onPing("hello");

    expect(first).not.toHaveBeenCalled();
    expect(second).toHaveBeenCalledWith("hello");
    expect(attach).toHaveBeenCalledTimes(1);
    unmount();
    expect(unlisten).toHaveBeenCalledTimes(1);
  });

  it("reports attach failures", async () => {
    const onAttachError = vi.fn();
    renderHook(() =>
      useBackendEvents(async () => Promise.reject(new Error("no runtime")), {}, onAttachError)
    );
    await waitFor(() => expect(onAttachError).toHaveBeenCalled());
  });
});
