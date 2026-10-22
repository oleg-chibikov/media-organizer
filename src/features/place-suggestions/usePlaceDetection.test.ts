import { act, renderHook } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { detectPlaceSuggestions } from "../../lib/api";
import type { HomeLocation, PlaceSuggestion } from "../../lib/types";
import { METADATA, RECORD } from "../../test/pipelineFixtures";
import { usePlaceDetection } from "./usePlaceDetection";

vi.mock("../../lib/api", () => ({ detectPlaceSuggestions: vi.fn() }));

const detect = vi.mocked(detectPlaceSuggestions);
const SUGGESTION = { id: "s1" } as PlaceSuggestion;
const SYDNEY = { latitude: -33.87, longitude: 151.21 } as HomeLocation;

interface DetectionProps {
  metadataBusy: boolean;
  enabled?: boolean;
  home?: HomeLocation | null;
}

function renderDetection(initial: DetectionProps) {
  const reportError = vi.fn();
  return renderHook(
    ({ metadataBusy, enabled = true, home = null }: DetectionProps) =>
      usePlaceDetection({
        rows: [RECORD],
        metadataByFileId: metadataBusy ? {} : { f1: METADATA },
        scanCompleted: true,
        metadataBusy,
        enabled,
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
    detect.mockResolvedValue([SUGGESTION]);
    const { result, rerender } = renderDetection({ metadataBusy: true });
    act(() => vi.advanceTimersByTime(2000));
    expect(detect).not.toHaveBeenCalled();

    rerender({ metadataBusy: false });
    act(() => vi.advanceTimersByTime(999));
    expect(detect).not.toHaveBeenCalled();
    await act(async () => vi.advanceTimersByTimeAsync(1));

    expect(detect).toHaveBeenCalledWith([RECORD], [METADATA]);
    expect(result.current.suggestions).toEqual([SUGGESTION]);
    rerender({ metadataBusy: false });
    act(() => vi.advanceTimersByTime(2000));
    expect(detect).toHaveBeenCalledTimes(1);
  });

  it("ignores a late answer from before a reset", async () => {
    let answer: (found: PlaceSuggestion[]) => void = () => undefined;
    detect.mockReturnValueOnce(new Promise((resolve) => (answer = resolve)));
    detect.mockResolvedValueOnce([]);
    const { result } = renderDetection({ metadataBusy: false });
    act(() => vi.advanceTimersByTime(1000));

    act(() => result.current.reset());
    await act(async () => answer([SUGGESTION]));

    expect(result.current.suggestions).toEqual([]);
    await act(async () => vi.advanceTimersByTimeAsync(1000));
    expect(detect).toHaveBeenCalledTimes(2);
  });

  it("asks again when home changes and drops the answer for the old home", async () => {
    let answer: (found: PlaceSuggestion[]) => void = () => undefined;
    detect.mockReturnValueOnce(new Promise((resolve) => (answer = resolve)));
    detect.mockResolvedValueOnce([SUGGESTION]);
    const { result, rerender } = renderDetection({ metadataBusy: false });
    act(() => vi.advanceTimersByTime(1000));

    rerender({ metadataBusy: false, home: SYDNEY });
    await act(async () => vi.advanceTimersByTimeAsync(1000));
    await act(async () => answer([]));

    expect(detect).toHaveBeenCalledTimes(2);
    expect(result.current.suggestions).toEqual([SUGGESTION]);
  });

  it("skips detection when files are renamed in place", () => {
    renderDetection({ metadataBusy: false, enabled: false });
    act(() => vi.advanceTimersByTime(2000));
    expect(detect).not.toHaveBeenCalled();
  });
});
