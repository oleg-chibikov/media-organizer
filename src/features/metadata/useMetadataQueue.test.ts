import { act, renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { MetadataResult } from "../../lib/types";
import { METADATA, RECORD } from "../../test/pipelineFixtures";
import { readMetadataForFiles } from "../../lib/api";
import { useMetadataQueue } from "./useMetadataQueue";

const pending = vi.hoisted(() => ({
  resolve: null as ((results: MetadataResult[]) => void) | null
}));

vi.mock("../../lib/api", () => ({
  readMetadataForFiles: vi.fn(
    () => new Promise<MetadataResult[]>((resolve) => (pending.resolve = resolve))
  )
}));

describe("useMetadataQueue", () => {
  it("drops a batch that finishes after a reset", async () => {
    const onMetadataRead = vi.fn();
    const rowByIdRef = { current: new Map([[RECORD.id, RECORD]]) };
    const { result, rerender } = renderHook(
      ({ rows }) =>
        useMetadataQueue({
          rows,
          rowByIdRef,
          concurrency: 1,
          queueCapacity: 10,
          reportError: vi.fn(),
          onMetadataRead
        }),
      { initialProps: { rows: [RECORD] } }
    );
    await waitFor(() => expect(result.current.activeMetadataRequests).toBe(1));
    const finishOldBatch = pending.resolve;

    act(() => {
      result.current.resetMetadata();
      rowByIdRef.current = new Map();
      rerender({ rows: [] });
    });
    await act(async () => finishOldBatch?.([METADATA]));

    expect(result.current.activeMetadataRequests).toBe(0);
    expect(result.current.metadataByFileId).toEqual({});
    expect(onMetadataRead).not.toHaveBeenCalled();
  });

  it("keeps a file's read error in its row", async () => {
    const reportError = vi.fn();
    const rowByIdRef = { current: new Map([[RECORD.id, RECORD]]) };
    const { result } = renderHook(() =>
      useMetadataQueue({
        rows: [RECORD],
        rowByIdRef,
        concurrency: 1,
        queueCapacity: 10,
        reportError,
        onMetadataRead: vi.fn()
      })
    );
    await waitFor(() => expect(result.current.activeMetadataRequests).toBe(1));

    const failed = { ...METADATA, metadata_status: "error" as const, error: "Timed out" };
    await act(async () => pending.resolve?.([failed]));

    expect(result.current.metadataByFileId[RECORD.id]).toEqual(failed);
    expect(reportError).not.toHaveBeenCalled();
  });

  it("waits for the resume before it reads more files", async () => {
    const rowByIdRef = { current: new Map([[RECORD.id, RECORD]]) };
    vi.mocked(readMetadataForFiles).mockClear();
    const { result, rerender } = renderHook(
      ({ rows }) =>
        useMetadataQueue({
          rows,
          rowByIdRef,
          concurrency: 1,
          queueCapacity: 10,
          reportError: vi.fn(),
          onMetadataRead: vi.fn()
        }),
      { initialProps: { rows: [] as (typeof RECORD)[] } }
    );

    act(() => result.current.setPaused(true));
    rerender({ rows: [RECORD] });
    expect(result.current.paused).toBe(true);
    expect(readMetadataForFiles).not.toHaveBeenCalled();

    act(() => result.current.setPaused(false));
    await waitFor(() => expect(result.current.activeMetadataRequests).toBe(1));
    expect(readMetadataForFiles).toHaveBeenCalledTimes(1);
  });

  it("removes only the given files' results", () => {
    const { result } = renderHook(() =>
      useMetadataQueue({
        rows: [],
        rowByIdRef: { current: new Map() },
        concurrency: 1,
        queueCapacity: 10,
        reportError: vi.fn(),
        onMetadataRead: vi.fn()
      })
    );
    const other = { ...METADATA, file_id: "f2" };
    act(() => result.current.addResults([METADATA, other]));

    act(() => result.current.removeResults([METADATA.file_id]));

    expect(result.current.metadataByFileId).toEqual({ f2: other });
  });
});
