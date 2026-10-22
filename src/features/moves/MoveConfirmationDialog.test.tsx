import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { MoveConfirmationSummary } from "../../lib/types";
import { MoveConfirmationDialog } from "./MoveConfirmationDialog";

afterEach(cleanup);

const SUMMARY = {
  ready_operations: 57,
  skipped_operations: 1,
  fallback_warnings: 5,
  suffix_applied: 0,
  gather_library: true,
  library_folders: []
} as unknown as MoveConfirmationSummary;

const WITH_EARLIER_DAY: MoveConfirmationSummary = {
  ...SUMMARY,
  library_folders: [
    { from: "D:\\Media\\2024\\3\\4\\Dubai", to: "D:\\Media\\2024\\3\\Dubai", files: 10 }
  ]
};

function renderDialog(
  stillChecking: boolean,
  renamesInPlace = false,
  groupByPlace = false,
  summary = SUMMARY
) {
  const onConfirm = vi.fn();
  const onCancel = vi.fn();
  const onGatherLibraryChange = vi.fn();
  render(
    <MoveConfirmationDialog
      summary={summary}
      destinationRoot={"D:\\Media"}
      renamesInPlace={renamesInPlace}
      stillChecking={stillChecking}
      groupByPlace={groupByPlace}
      gathering={false}
      onGatherLibraryChange={onGatherLibraryChange}
      onConfirm={onConfirm}
      onCancel={onCancel}
    />
  );
  return { onConfirm, onCancel, onGatherLibraryChange };
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
    expect(screen.queryByRole("checkbox")).not.toBeInTheDocument();

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

  it("says grouped files wait for every date", () => {
    renderDialog(true, false, true);

    expect(screen.getByText(/move once every date is read/)).toBeInTheDocument();
    expect(screen.queryByText(/ready now/)).not.toBeInTheDocument();
  });

  it("asks to rename when files stay in their folders", () => {
    const { onConfirm } = renderDialog(false, true);

    expect(screen.getByRole("dialog", { name: "Rename 57 files?" })).toBeInTheDocument();
    expect(screen.getByText(/stays in its folder/)).toBeInTheDocument();
    expect(screen.queryByText("D:\\Media")).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Rename 57 files" }));
    expect(onConfirm).toHaveBeenCalledTimes(1);
  });

  it("lists earlier days that move too, and lets the person leave them", () => {
    const { onGatherLibraryChange } = renderDialog(false, false, false, WITH_EARLIER_DAY);

    expect(screen.getByText("2024\\3\\4\\Dubai")).toBeInTheDocument();
    expect(screen.getByText("2024\\3\\Dubai")).toBeInTheDocument();
    expect(screen.getByText(/10 files/)).toBeInTheDocument();

    fireEvent.click(screen.getByRole("checkbox", { name: /earlier days/ }));
    expect(onGatherLibraryChange).toHaveBeenCalledWith(false);
  });
});
