import { act, renderHook } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { AUTO_SIZE_INTERVAL_MS, useColumnSizing } from "./useColumnSizing";

const COLUMNS = [{ key: "name", label: "Name" }] as const;

afterEach(() => vi.useRealTimers());

describe("useColumnSizing", () => {
  it("measures a burst of row updates once", () => {
    vi.useFakeTimers();
    const measure = vi.fn(() => ({ name: 200 }));
    const { result, rerender } = renderHook(
      ({ rowCount }) =>
        useColumnSizing({
          rowCount,
          columns: COLUMNS,
          defaultColumnSizing: { name: 50 },
          onAutoSizeColumns: () => measure()
        }),
      { initialProps: { rowCount: 1 } }
    );
    rerender({ rowCount: 2 });
    rerender({ rowCount: 3 });
    expect(measure).not.toHaveBeenCalled();

    act(() => vi.advanceTimersByTime(AUTO_SIZE_INTERVAL_MS));

    expect(measure).toHaveBeenCalledTimes(1);
    expect(result.current.columnSizing).toEqual({ name: 200 });
  });

  it("keeps a column the person resized by hand", () => {
    vi.useFakeTimers();
    const { result } = renderHook(() =>
      useColumnSizing({
        rowCount: 1,
        columns: COLUMNS,
        defaultColumnSizing: { name: 50 },
        onAutoSizeColumns: () => ({ name: 200 })
      })
    );

    act(() => result.current.markColumnAsResized("name"));
    act(() => vi.advanceTimersByTime(AUTO_SIZE_INTERVAL_MS));

    expect(result.current.columnSizing).toEqual({ name: 50 });
  });
});
