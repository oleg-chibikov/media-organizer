import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { EmptyFileList } from "./EmptyFileList";

afterEach(cleanup);

describe("EmptyFileList", () => {
  it("explains the first step and starts it", () => {
    const onChooseFolder = vi.fn();
    render(
      <EmptyFileList
        selectedFolder={null}
        isScanning={false}
        chooseDisabled={false}
        onChooseFolder={onChooseFolder}
      />
    );

    fireEvent.click(screen.getByRole("button", { name: "Choose Folder" }));
    expect(onChooseFolder).toHaveBeenCalledTimes(1);
    expect(screen.getByText(/You can undo a run in History/)).toBeInTheDocument();
  });

  it("says when a scanned folder has nothing to sort", () => {
    render(
      <EmptyFileList
        selectedFolder={"C:\\in"}
        isScanning={false}
        chooseDisabled={false}
        onChooseFolder={vi.fn()}
      />
    );

    expect(screen.getByText("No photos or videos here")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Choose Another Folder" })).toBeInTheDocument();
  });
});
