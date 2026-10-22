import { renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { PlaceSuggestion } from "../../lib/types";
import { OUTING, TRIP } from "./testSuggestions";
import type { PlaceDetectionOptions } from "./usePlaceDetection";
import { usePlaceSuggestions } from "./usePlaceSuggestions";

const detection = vi.hoisted(() => ({ settled: false, suggestions: [] as PlaceSuggestion[] }));

vi.mock("./usePlaceDetection", () => ({
  usePlaceDetection: () => ({
    detection: { suggestions: detection.suggestions, file_places: {}, status: { kind: "found" } },
    locating: !detection.settled,
    settled: detection.settled,
    progress: { read: 1, total: 1 },
    reset: vi.fn()
  })
}));

function renderSuggestions(groupByPlace: boolean) {
  const onApply = vi.fn();
  const options = {} as PlaceDetectionOptions;
  const hook = renderHook(
    (group: boolean) => usePlaceSuggestions({ ...options, groupByPlace: group, onApply }),
    {
      initialProps: groupByPlace
    }
  );
  return { ...hook, onApply };
}

describe("usePlaceSuggestions", () => {
  it("waits for detection, then applies what it found once", () => {
    detection.settled = false;
    detection.suggestions = [TRIP, OUTING];
    const { rerender, onApply } = renderSuggestions(true);
    expect(onApply).not.toHaveBeenCalled();

    detection.settled = true;
    rerender(true);
    detection.suggestions = [TRIP, OUTING];
    rerender(true);

    expect(onApply).toHaveBeenCalledExactlyOnceWith([TRIP, OUTING]);
  });

  it("applies no place folders while grouping is off", () => {
    detection.settled = false;
    detection.suggestions = [TRIP];
    const { onApply } = renderSuggestions(false);

    expect(onApply).toHaveBeenCalledExactlyOnceWith([]);
  });
});
