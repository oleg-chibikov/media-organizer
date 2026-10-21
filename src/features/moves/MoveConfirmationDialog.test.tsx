import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { MoveConfirmationSummary } from "../../lib/types";
import { MoveConfirmationDialog } from "./MoveConfirmationDialog";

afterEach(cleanup);

const SUMMARY = {
  ready_operations: 57,
  skipped_operations: 2,
  fallback_warnings: 5,
  suffix_applied: 0
} as MoveConfirmationSummary;

describe("MoveConfirmationDialog", () => {
  it("names the destination and what happens to unusual files", () => {
    const onConfirm = vi.fn();
    const onCancel = vi.fn();
    render(
      <MoveConfirmationDialog
        summary={SUMMARY}
        destinationRoot={"D:\\Media"}
        onConfirm={onConfirm}
        onCancel={onCancel}
      />
    );

    expect(screen.getByRole("dialog", { name: "Move 57 files?" })).toBeInTheDocument();
    expect(screen.getByText("D:\\Media")).toBeInTheDocument();
    expect(screen.getByText(/5 files use the date the file last changed/)).toBeInTheDocument();
    expect(screen.getByText(/2 files stay where they are/)).toBeInTheDocument();
    expect(screen.queryByText(/number added/)).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    fireEvent.click(screen.getByRole("button", { name: "Move 57 files" }));
    expect(onCancel).toHaveBeenCalledTimes(1);
    expect(onConfirm).toHaveBeenCalledTimes(1);
  });
});
