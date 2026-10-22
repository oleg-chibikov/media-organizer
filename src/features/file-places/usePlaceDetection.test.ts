import { act, renderHook } from "@testing-library/react";
import { FileIdMap } from "../../lib/fileIdMap";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { detectFilePlaces } from "../../lib/api";
import type { FilePlace, HomeLocation, PlaceDetection } from "../../lib/types";
import { METADATA, RECORD } from "../../test/pipelineFixtures";
import { usePlaceDetection } from "./usePlaceDetection";

vi.mock("../../lib/api", () => ({ detectFilePlaces: vi.fn() }));

const detect = vi.mocked(detectFilePlaces);
const KYOTO: Record<string, FilePlace> = { f1: { place: "Kyoto", country: "Japan", home: "away" } };
const found = (filePlaces: Record<string, FilePlace>): PlaceDetection => ({
  file_places: filePlaces,
  status: { kind: Object.keys(filePlaces).length > 0 ? "found" : "no_gps" }
});
const SYDNEY = { latitude: -33.87, longitude: 151.21 } as HomeLocation;

interface DetectionProps {
  metadataBusy: boolean;
  home?: HomeLocation | null;
}

function renderDetection(initial: DetectionProps) {
  const reportError = vi.fn();
  return renderHook(
    ({ metadataBusy, home = null }: DetectionProps) =>
      usePlaceDetection({
        selectedFolder: "C:\\in",
        scanIdRef: { current: 1 },
        rows: [RECORD],
        metadataByFileId: FileIdMap.from(metadataBusy ? {} : { f1: METADATA }),
        scanCompleted: true,
        scanCancelled: false,
        metadataBusy,
        home,
        reportError
      }),
    { initialProps: initial }
  );
}

beforeEach(() => vi.useFakeTimers());
afterEach(() => {
  vi.useRealTimers();
  detect.mockReset();
});

describe("usePlaceDetection", () => {
  it("runs once, a second after metadata finishes", async () => {
    detect.mockResolvedValue(found(KYOTO));
    const { result, rerender } = renderDetection({ metadataBusy: true });
    act(() => vi.advanceTimersByTime(2000));
    expect(detect).not.toHaveBeenCalled();

    rerender({ metadataBusy: false });
    act(() => vi.advanceTimersByTime(999));
    expect(detect).not.toHaveBeenCalled();
    await act(async () => vi.advanceTimersByTimeAsync(1));

    expect(detect).toHaveBeenCalledWith(1, [RECORD.id], false);
    expect(result.current.detection?.file_places).toEqual(KYOTO);
    rerender({ metadataBusy: false });
    act(() => vi.advanceTimersByTime(2000));
    expect(detect).toHaveBeenCalledTimes(1);
  });

  it("ignores a late answer from before a reset", async () => {
    let answer: (detection: PlaceDetection) => void = () => undefined;
    detect.mockReturnValueOnce(new Promise((resolve) => (answer = resolve)));
    detect.mockResolvedValueOnce(found({}));
    const { result } = renderDetection({ metadataBusy: false });
    act(() => vi.advanceTimersByTime(1000));

    act(() => result.current.reset());
    await act(async () => answer(found(KYOTO)));

    expect(result.current.detection).toBeNull();
    await act(async () => vi.advanceTimersByTimeAsync(1000));
    expect(detect).toHaveBeenCalledTimes(2);
  });

  it("asks again when home changes and drops the answer for the old home", async () => {
    let answer: (detection: PlaceDetection) => void = () => undefined;
    detect.mockReturnValueOnce(new Promise((resolve) => (answer = resolve)));
    detect.mockResolvedValueOnce(found(KYOTO));
    const { result, rerender } = renderDetection({ metadataBusy: false });
    act(() => vi.advanceTimersByTime(1000));

    rerender({ metadataBusy: false, home: SYDNEY });
    await act(async () => vi.advanceTimersByTimeAsync(1000));
    await act(async () => answer(found({})));

    expect(detect).toHaveBeenCalledTimes(2);
    expect(result.current.detection?.file_places).toEqual(KYOTO);
  });

  it("reports locating with metadata progress until the answer lands", async () => {
    detect.mockResolvedValue(found({}));
    const { result, rerender } = renderDetection({ metadataBusy: true });
    expect(result.current.locating).toBe(true);
    expect(result.current.progress).toEqual({ read: 0, total: 1 });

    rerender({ metadataBusy: false });
    expect(result.current.progress).toEqual({ read: 1, total: 1 });
    await act(async () => vi.advanceTimersByTimeAsync(1000));

    expect(result.current.locating).toBe(false);
    expect(result.current.detection?.status).toEqual({ kind: "no_gps" });
  });
});
