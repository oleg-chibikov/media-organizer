import { act, renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { FileIdMap } from "../../lib/fileIdMap";
import { pauseMoves, resumeMoves, startMoves, type MoveListeners } from "../../lib/api";
import { METADATA, PLAN, RECORD } from "../../test/pipelineFixtures";
import { useMoveExecution } from "./useMoveExecution";

const listeners = vi.hoisted(() => ({ move: null as MoveListeners | null }));

vi.mock("../../lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("../../lib/api")>()),
  startMoves: vi.fn(async () => undefined),
  pauseMoves: vi.fn(async () => undefined),
  resumeMoves: vi.fn(async () => undefined),
  getMoveConfirmationSummary: vi.fn(async () => ({ ready_operations: 1 })),
  attachMoveListeners: vi.fn(async (handlers: MoveListeners) => ((listeners.move = handlers), []))
}));

function renderExecution() {
  const reportError = vi.fn();
  const rowByIdRef = { current: new Map([[RECORD.id, RECORD]]) };
  const scanIdRef = { current: 1 };
  const hook = renderHook(() =>
    useMoveExecution({
      stillChecking: false,
      filesFound: true,
      selectedFolder: "C:\\in",
      rows: [RECORD],
      rowByIdRef,
      scanIdRef,
      metadataByFileId: FileIdMap.from({ [RECORD.id]: METADATA }),
      movePlansByFileId: FileIdMap.from({ [RECORD.id]: PLAN }),
      reportError
    })
  );
  return { ...hook, reportError };
}

describe("useMoveExecution", () => {
  it("starts, pauses and resumes the backend run, then shows it done", async () => {
    const { result, reportError } = renderExecution();

    act(() => result.current.runExecutionAction());
    await waitFor(() => expect(result.current.confirmation.summary).not.toBeNull());
    act(() => result.current.confirmation.confirm());
    expect(startMoves).toHaveBeenCalledWith(1);
    expect(result.current.execution).toMatchObject({ confirmed: true, executing: true });

    act(() => result.current.runExecutionAction());
    expect(pauseMoves).toHaveBeenCalledWith(1);
    expect(result.current.execution.paused).toBe(true);
    act(() => result.current.runExecutionAction());
    expect(resumeMoves).toHaveBeenCalledWith(1);

    act(() => listeners.move?.onDone({ scan_id: 1, removed_selected_folder: false, error: "x" }));
    expect(result.current.execution).toMatchObject({ executing: false, completed: true });
    expect(reportError).toHaveBeenCalledWith("x");
  });
});
