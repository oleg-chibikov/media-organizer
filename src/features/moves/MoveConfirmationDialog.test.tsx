import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { MoveConfirmationSummary } from "../../lib/types";
import { MoveConfirmationDialog } from "./MoveConfirmationDialog";

afterEach(cleanup);

const SUMMARY = {
  ready_operations: 57,
  skipped_operations: 1,
  fallback_warnings: 5,
  suffix_applied: 0
} as MoveConfirmationSummary;

function renderDialog(stillChecking: boolean) {
  const onConfirm = vi.fn();
  const onCancel = vi.fn();
  render(
    <MoveConfirmationDialog
      summary={SUMMARY}
      destinationRoot={"D:\\Media"}
      stillChecking={stillChecking}
      onConfirm={onConfirm}
      onCancel={onCancel}
    />
  );
  return { onConfirm, onCancel };
}

describe("MoveConfirmationDialog", () => {
  it("names the destination and what happens to unusual files", () => {
    const { onConfirm, onCancel } = renderDialog(false);

    expect(screen.getByRole("dialog", { name: "Move 57 files?" })).toBeInTheDocument();
    expect(screen.getByText("D:\\Media")).toBeInTheDocument();
    expect(screen.getByText(/5 files use the date they last changed/)).toBeInTheDocument();
    expect(screen.getByText(/1 file stays where it is/)).toBeInTheDocument();
    expect(screen.queryByText(/number added/)).not.toBeInTheDocument();
    expect(screen.queryByText(/still being checked/)).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    fireEvent.click(screen.getByRole("button", { name: "Move 57 files" }));
    expect(onCancel).toHaveBeenCalledTimes(1);
    expect(onConfirm).toHaveBeenCalledTimes(1);
  });

  it("offers every file while files are still being checked", () => {
    const { onConfirm } = renderDialog(true);

    expect(screen.getByRole("dialog", { name: "Move all files?" })).toBeInTheDocument();
    expect(screen.getByText(/57 files ready now/)).toBeInTheDocument();
    expect(screen.getByText(/still being checked move too/)).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Move all files" }));
    expect(onConfirm).toHaveBeenCalledTimes(1);
  });
});
