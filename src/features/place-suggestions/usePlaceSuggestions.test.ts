import { act, renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { freshChoice, type PlaceChoice } from "./placeChoice";
import { OUTING, TRIP } from "./testSuggestions";
import type { PlaceDetectionOptions } from "./usePlaceDetection";
import { usePlaceSuggestions } from "./usePlaceSuggestions";

vi.mock("./usePlaceDetection", () => ({
  usePlaceDetection: () => ({ suggestions: [TRIP, OUTING], reset: vi.fn() })
}));

function renderSuggestions(locked = false) {
  const onApply = vi.fn();
  const detection = {} as PlaceDetectionOptions;
  const hook = renderHook(() => usePlaceSuggestions({ ...detection, locked, onApply }));
  return { ...hook, onApply };
}

describe("usePlaceSuggestions", () => {
  it("hides dismissed suggestions", () => {
    const { result } = renderSuggestions();
    expect(result.current.pending).toEqual([TRIP, OUTING]);
    act(() => result.current.dismiss());
    expect(result.current.pending).toEqual([]);
  });

  it("hides suggestions once moves are confirmed", () => {
    expect(renderSuggestions(true).result.current.pending).toEqual([]);
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
