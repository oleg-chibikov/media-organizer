import { act, renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { freshChoice, type PlaceChoice } from "./placeChoice";
import { OUTING, TRIP } from "./testSuggestions";
import type { PlaceDetectionOptions } from "./usePlaceDetection";
import { usePlaceSuggestions } from "./usePlaceSuggestions";

vi.mock("./usePlaceDetection", () => ({
  usePlaceDetection: () => ({
    detection: { suggestions: [TRIP, OUTING], file_places: {}, status: { kind: "found" } },
    locating: false,
    progress: { read: 1, total: 1 },
    reset: vi.fn()
  })
}));

function renderSuggestions(locked = false) {
  const onApply = vi.fn();
  const detection = {} as PlaceDetectionOptions;
  const hook = renderHook(() => usePlaceSuggestions({ ...detection, locked, onApply }));
  return { ...hook, onApply };
}

describe("usePlaceSuggestions", () => {
  it("hides the banner once dismissed and keeps the suggestions for the review", () => {
    const { result } = renderSuggestions();
    expect(result.current.bannerVisible).toBe(true);
    act(() => result.current.dismiss());
    expect(result.current.bannerVisible).toBe(false);
    act(() => result.current.openReview());
    expect(result.current.reviewOpen).toBe(true);
    expect(result.current.suggestions).toEqual([TRIP, OUTING]);
  });

  it("hides suggestions once moves are confirmed", () => {
    const { result } = renderSuggestions(true);
    expect(result.current.suggestions).toEqual([]);
    expect(result.current.bannerVisible).toBe(false);
  });

  it("applies the used suggestions and remembers every choice", () => {
    const { result, onApply } = renderSuggestions();
    const choices: Record<string, PlaceChoice> = {
      [TRIP.id]: freshChoice(TRIP),
      [OUTING.id]: { ...freshChoice(OUTING), use: false }
    };
    act(() => result.current.openReview());
    act(() => result.current.apply(choices));

    expect(onApply).toHaveBeenCalledWith([TRIP]);
    expect(result.current.choices).toEqual(choices);
    expect(result.current.reviewOpen).toBe(false);
  });
});
