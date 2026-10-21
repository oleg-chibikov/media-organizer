import { act, renderHook } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { RECORD } from "../../test/pipelineFixtures";
import { SCAN_ROW_FLUSH_MS, useScanRows } from "./useScanRows";

afterEach(() => {
  vi.useRealTimers();
});

describe("useScanRows", () => {
  it("indexes rows at once and appends them in one batch", () => {
    vi.useFakeTimers();
    const { result } = renderHook(() => useScanRows());
    const second = { ...RECORD, id: "f2" };

    act(() => {
      expect(result.current.addRow(RECORD)).toBe(true);
      expect(result.current.addRow(second)).toBe(true);
      expect(result.current.addRow(RECORD)).toBe(false);
    });

    expect(result.current.rows).toEqual([]);
    expect(result.current.rowByIdRef.current.get("f2")).toBe(second);
    act(() => vi.advanceTimersByTime(SCAN_ROW_FLUSH_MS));
    expect(result.current.rows).toEqual([RECORD, second]);
  });

  it("flushes pending rows on demand and forgets them on reset", () => {
    const { result } = renderHook(() => useScanRows());

    act(() => {
      result.current.addRow(RECORD);
      result.current.flushRows();
    });
    expect(result.current.rows).toEqual([RECORD]);

    act(() => result.current.resetRows());
    expect(result.current.rows).toEqual([]);
    expect(result.current.rowByIdRef.current.size).toBe(0);
  });
});
