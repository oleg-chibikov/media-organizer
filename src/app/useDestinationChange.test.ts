import { renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { IDLE_MOVE_EXECUTION } from "../features/moves";
import { METADATA, RECORD } from "../test/pipelineFixtures";
import { useDestinationChange } from "./useDestinationChange";

type Options = Parameters<typeof useDestinationChange>[0];

function options(confirmed: boolean, rows = [RECORD]) {
  const onChangedRef: { current: (() => void) | null } = { current: null };
  const applyChange = vi.fn(async (_operation, context: { onChanged: () => void }) => {
    onChangedRef.current = context.onChanged;
  });
  const value = {
    destination: { applyChange },
    scan: { selectedFolder: "C:\\in", rows },
    metadataByFileId: { [RECORD.id]: METADATA },
    planning: { clearPlans: vi.fn(), requestPlans: vi.fn() },
    moves: { execution: { ...IDLE_MOVE_EXECUTION, confirmed }, resetMovesForReplan: vi.fn() }
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

  it("replans with the rows present when the change lands", () => {
    const first = options(false, []);
    const { result, rerender } = renderHook((props: Options) => useDestinationChange(props), {
      initialProps: first.value
    });
    result.current.clearRoot();
    const latest = options(false);
    rerender(latest.value);

    first.onChangedRef.current?.();

    expect(latest.value.planning.requestPlans).toHaveBeenCalledWith([RECORD], [METADATA]);
  });
});
