import { act, renderHook } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { getMoveConfirmationSummary } from "../../lib/api";
import type { MoveConfirmationSummary } from "../../lib/types";
import { METADATA, PLAN, RECORD } from "../../test/pipelineFixtures";
import { useMoveConfirmation } from "./useMoveConfirmation";

vi.mock("../../lib/api", () => ({ getMoveConfirmationSummary: vi.fn() }));

const SUMMARY = { ready_operations: 7, skipped_operations: 2 } as MoveConfirmationSummary;

function renderConfirmation() {
  const onConfirmed = vi.fn();
  const rowByIdRef = { current: new Map([[RECORD.id, RECORD]]) };
  const { result } = renderHook(() =>
    useMoveConfirmation({
      rows: [RECORD],
      rowByIdRef,
      metadataByFileId: { [RECORD.id]: METADATA },
      movePlansByFileId: { [RECORD.id]: PLAN },
      reportError: vi.fn(),
      onConfirmed
    })
  );
  return { confirm: result.current, onConfirmed, rowByIdRef };
}

afterEach(() => vi.restoreAllMocks());

describe("useMoveConfirmation", () => {
  it("asks with the backend summary and starts only after a yes", async () => {
    vi.mocked(getMoveConfirmationSummary).mockResolvedValue(SUMMARY);
    const ask = vi.spyOn(window, "confirm").mockReturnValueOnce(false).mockReturnValueOnce(true);
    const { confirm, onConfirmed } = renderConfirmation();

    await act(confirm);
    expect(onConfirmed).not.toHaveBeenCalled();
    await act(confirm);

    expect(getMoveConfirmationSummary).toHaveBeenCalledWith([RECORD], [METADATA], [PLAN]);
    expect(ask).toHaveBeenCalledWith(expect.stringContaining("Ready operations: 7"));
    expect(onConfirmed).toHaveBeenCalledTimes(1);
  });

  it("doesn't ask when a new scan started while the summary loaded", async () => {
    const { confirm, onConfirmed, rowByIdRef } = renderConfirmation();
    vi.mocked(getMoveConfirmationSummary).mockImplementation(async () => {
      rowByIdRef.current = new Map();
      return SUMMARY;
    });
    const ask = vi.spyOn(window, "confirm");

    await act(confirm);

    expect(ask).not.toHaveBeenCalled();
    expect(onConfirmed).not.toHaveBeenCalled();
  });
});
