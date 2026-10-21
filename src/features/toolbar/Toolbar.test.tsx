import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { Toolbar } from "./Toolbar";

afterEach(cleanup);

describe("Toolbar", () => {
  it("shows the folder state and the scan controls", () => {
    const onCancelScan = vi.fn();
    render(
      <Toolbar
        selectedFolder={null}
        folderError="Selected folder is inside a Processed path."
        isScanning={true}
        rootMediaFolder={"D:\\Media"}
        historyMenu={<button type="button">History</button>}
        onChooseFolder={vi.fn()}
        onCancelScan={onCancelScan}
      />
    );

    expect(screen.getByText("No folder selected")).toBeInTheDocument();
    expect(screen.getByText("Selected folder is inside a Processed path.")).toBeInTheDocument();
    expect(screen.getByText("Root media: D:\\Media")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "History" })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Stop Scan" }));
    expect(onCancelScan).toHaveBeenCalledTimes(1);
  });
});
