import { renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { FilePlace } from "../../lib/types";
import type { PlaceDetectionOptions } from "./usePlaceDetection";
import { useFilePlaces } from "./useFilePlaces";

const KYOTO: Record<string, FilePlace> = { f1: { place: "Kyoto", country: "Japan", home: "away" } };
const detection = vi.hoisted(() => ({ settled: false, places: {} as Record<string, FilePlace> }));

vi.mock("./usePlaceDetection", () => ({
  usePlaceDetection: () => ({
    detection: { file_places: detection.places, status: { kind: "found" } },
    locating: !detection.settled,
    settled: detection.settled,
    progress: { read: 1, total: 1 },
    reset: vi.fn()
  })
}));

function renderPlaces(groupByPlace: boolean) {
  const onApply = vi.fn();
  const options = {} as PlaceDetectionOptions;
  const hook = renderHook(
    (group: boolean) => useFilePlaces({ ...options, groupByPlace: group, onApply }),
    { initialProps: groupByPlace }
  );
  return { ...hook, onApply };
}

describe("useFilePlaces", () => {
  it("waits for detection, then applies what it found once", () => {
    detection.settled = false;
    detection.places = KYOTO;
    const { rerender, onApply } = renderPlaces(true);
    expect(onApply).not.toHaveBeenCalled();

    detection.settled = true;
    rerender(true);
    rerender(true);

    expect(onApply).toHaveBeenCalledExactlyOnceWith(KYOTO);
  });

  it("applies no places while grouping is off, but still shows them", () => {
    detection.settled = false;
    detection.places = KYOTO;
    const { result, onApply } = renderPlaces(false);

    expect(onApply).toHaveBeenCalledExactlyOnceWith({});
    expect(result.current.filePlaces).toBe(KYOTO);
  });
});
