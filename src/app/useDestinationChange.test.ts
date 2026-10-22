import { renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { IDLE_MOVE_EXECUTION } from "../features/moves";
import type { DestinationSettings } from "../lib/types";
import { RECORD } from "../test/pipelineFixtures";
import { useDestinationChange } from "./useDestinationChange";

type Options = Parameters<typeof useDestinationChange>[0];

function options(confirmed: boolean, rows = [RECORD]) {
  type OnChanged = (settings: DestinationSettings) => void;
  const onChangedRef: { current: OnChanged | null } = { current: null };
  const applyChange = vi.fn(async (_operation, context: { onChanged: OnChanged }) => {
    onChangedRef.current = context.onChanged;
  });
  const value = {
    destination: { applyChange },
    scan: { selectedFolder: "C:\\in", rows },
    planning: { clearPlans: vi.fn() },
    moves: { execution: { ...IDLE_MOVE_EXECUTION, confirmed }, resetMovesForReplan: vi.fn() },
    placeSuggestions: { reset: vi.fn() }
  } as unknown as Options;
  return { value, applyChange, onChangedRef };
}

describe("useDestinationChange", () => {
  it("locks settings once moves are confirmed", () => {
    const { value, applyChange } = options(true);
    const { result } = renderHook(() => useDestinationChange(value));

    result.current.clearRoot();

    expect(result.current.locked).toBe(true);
    expect(applyChange).toHaveBeenCalledWith(
      expect.any(Function),
      expect.objectContaining({ locked: true })
    );
  });

  it("replans the session that is on screen when the change lands", () => {
    const first = options(false, []);
    const { result, rerender } = renderHook((props: Options) => useDestinationChange(props), {
      initialProps: first.value
    });
    result.current.clearRoot();
    const latest = options(false);
    rerender(latest.value);

    first.onChangedRef.current?.({ group_by_place: true } as DestinationSettings);

    expect(first.value.planning.clearPlans).not.toHaveBeenCalled();
    expect(latest.value.planning.clearPlans).toHaveBeenCalledOnce();
    expect(latest.value.moves.resetMovesForReplan).toHaveBeenCalledOnce();
    expect(latest.value.placeSuggestions.reset).toHaveBeenCalledOnce();
  });
});
