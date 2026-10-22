import { act, renderHook } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { detectFilePlaces } from "../../lib/api";
import { FileIdMap } from "../../lib/fileIdMap";
import type { MetadataResult, PlaceDetection } from "../../lib/types";
import { METADATA, RECORD } from "../../test/pipelineFixtures";
import { usePlacePreview, type PreviewSource } from "./usePlacePreview";

vi.mock("../../lib/api", () => ({ detectFilePlaces: vi.fn() }));

const detect = vi.mocked(detectFilePlaces);
const FOUND: PlaceDetection = { file_places: {}, status: { kind: "no_gps" } };

function source(metadata: Record<string, MetadataResult>): PreviewSource {
  return { scanIdRef: { current: 1 }, rows: [RECORD], metadataByFileId: FileIdMap.from(metadata) };
}

function renderPreview(active: boolean, latest: PreviewSource) {
  const onPreview = vi.fn();
  const latestRef = { current: latest };
  const hook = renderHook(
    ({ on }: { on: boolean }) =>
      usePlacePreview({ active: on, resetKey: "0", latestRef, onPreview }),
    { initialProps: { on: active } }
  );
  return { ...hook, latestRef, onPreview };
}

beforeEach(() => vi.useFakeTimers());
afterEach(() => {
  vi.useRealTimers();
  detect.mockReset();
});

describe("usePlacePreview", () => {
  it("asks every 2 s while new dates land, as a preview", async () => {
    detect.mockResolvedValue(FOUND);
    const { latestRef, onPreview } = renderPreview(true, source({ f1: METADATA }));

    await act(async () => vi.advanceTimersByTimeAsync(2000));
    expect(detect).toHaveBeenCalledExactlyOnceWith(1, [RECORD.id], true);
    expect(onPreview).toHaveBeenCalledWith(FOUND);

    await act(async () => vi.advanceTimersByTimeAsync(2000));
    expect(detect).toHaveBeenCalledTimes(1);
    latestRef.current = source({ f1: METADATA });
    await act(async () => vi.advanceTimersByTimeAsync(2000));
    expect(detect).toHaveBeenCalledTimes(2);
  });

  it("waits for the first date", async () => {
    renderPreview(true, source({}));
    await act(async () => vi.advanceTimersByTimeAsync(4000));
    expect(detect).not.toHaveBeenCalled();
  });

  it("drops a late answer once the final detection takes over", async () => {
    let answer: (detection: PlaceDetection) => void = () => undefined;
    detect.mockReturnValueOnce(new Promise((resolve) => (answer = resolve)));
    const { rerender, onPreview } = renderPreview(true, source({ f1: METADATA }));
    await act(async () => vi.advanceTimersByTimeAsync(2000));

    rerender({ on: false });
    await act(async () => answer(FOUND));

    expect(onPreview).not.toHaveBeenCalled();
  });
});
