import { act, renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { setMetadataPaused } from "../../lib/api";
import { METADATA, RECORD } from "../../test/pipelineFixtures";
import { useMetadataReads } from "./useMetadataReads";

vi.mock("../../lib/api", () => ({ setMetadataPaused: vi.fn(async () => undefined) }));

function renderReads(scanId: number | null = 1) {
  const onMetadataRead = vi.fn();
  const scanIdRef = { current: scanId };
  const rowByIdRef = { current: new Map([[RECORD.id, RECORD]]) };
  const hook = renderHook(
    ({ id }) =>
      useMetadataReads({ scanId: id, scanIdRef, rowByIdRef, reportError: vi.fn(), onMetadataRead }),
    { initialProps: { id: scanId } }
  );
  return { ...hook, onMetadataRead, scanIdRef };
}

describe("useMetadataReads", () => {
  it("keeps the current scan's dates and hands them to planning when reading ends", () => {
    const { result, onMetadataRead } = renderReads();
    expect(result.current.reading).toBe(true);

    act(() => {
      result.current.events.onResults({ scan_id: 1, results: [METADATA] });
      result.current.events.onFinished({ scan_id: 1, cancelled: false });
    });

    expect(result.current.metadataByFileId.get(RECORD.id)).toEqual(METADATA);
    expect(onMetadataRead).toHaveBeenCalledWith([RECORD], [METADATA]);
    expect(result.current.reading).toBe(false);
  });

  it("drops the dates and the end of a scan it replaced", () => {
    const { result, onMetadataRead } = renderReads();

    act(() => {
      result.current.events.onResults({ scan_id: 0, results: [METADATA] });
      result.current.events.onFinished({ scan_id: 0, cancelled: true });
    });

    expect(result.current.metadataByFileId.size).toBe(0);
    expect(onMetadataRead).not.toHaveBeenCalled();
    expect(result.current.reading).toBe(true);
  });

  it("reads nothing before a scan starts", () => {
    expect(renderReads(null).result.current.reading).toBe(false);
  });

  it("asks the backend to hold reads while moves are paused", () => {
    const { result } = renderReads();
    act(() => result.current.setPaused(true));
    expect(setMetadataPaused).toHaveBeenCalledWith(true);
  });
});
