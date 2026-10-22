import { act, renderHook } from "@testing-library/react";
import { FileIdMap } from "../../lib/fileIdMap";
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
      scanIdRef: { current: 1 },
      metadataByFileId: FileIdMap.from({ [RECORD.id]: METADATA }),
      movePlansByFileId: FileIdMap.from({ [RECORD.id]: PLAN }),
      reportError: vi.fn(),
      onConfirmed
    })
  );
  return { result, onConfirmed, rowByIdRef };
}

afterEach(() => vi.restoreAllMocks());

describe("useMoveConfirmation", () => {
  it("shows the backend summary and starts only after a yes", async () => {
    vi.mocked(getMoveConfirmationSummary).mockResolvedValue(SUMMARY);
    const { result, onConfirmed } = renderConfirmation();

    await act(() => result.current.request());
    expect(getMoveConfirmationSummary).toHaveBeenCalledWith(1, [RECORD.id]);
    expect(result.current.summary).toBe(SUMMARY);
    act(() => result.current.cancel());
    expect(result.current.summary).toBeNull();
    expect(onConfirmed).not.toHaveBeenCalled();

    await act(() => result.current.request());
    act(() => result.current.confirm());
    expect(result.current.summary).toBeNull();
    expect(onConfirmed).toHaveBeenCalledTimes(1);
  });

  it("doesn't ask when a new scan started while the summary loaded", async () => {
    const { result, onConfirmed, rowByIdRef } = renderConfirmation();
    vi.mocked(getMoveConfirmationSummary).mockImplementation(async () => {
      rowByIdRef.current = new Map();
      return SUMMARY;
    });

    await act(() => result.current.request());

    expect(result.current.summary).toBeNull();
    expect(onConfirmed).not.toHaveBeenCalled();
  });

  it("doesn't start when a new scan started while the dialog was open", async () => {
    vi.mocked(getMoveConfirmationSummary).mockResolvedValue(SUMMARY);
    const { result, onConfirmed, rowByIdRef } = renderConfirmation();

    await act(() => result.current.request());
    rowByIdRef.current = new Map();
    act(() => result.current.confirm());

    expect(onConfirmed).not.toHaveBeenCalled();
  });
});
