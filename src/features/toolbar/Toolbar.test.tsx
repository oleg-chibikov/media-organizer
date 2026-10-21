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
        movesRunning={false}
        rootMediaFolder={"D:\\Media"}
        historyMenu={<button type="button">History</button>}
        onChooseFolder={vi.fn()}
        onCancelScan={onCancelScan}
      />
    );

    expect(screen.getByText("No folder selected")).toBeInTheDocument();
    expect(screen.getByRole("alert")).toHaveTextContent(
      "Selected folder is inside a Processed path."
    );
    expect(screen.getByText("Root media: D:\\Media")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "History" })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Stop Scan" }));
    expect(onCancelScan).toHaveBeenCalledTimes(1);
  });

  it("blocks a new folder while moves are running", () => {
    render(
      <Toolbar
        selectedFolder={"C:\\in"}
        folderError={null}
        isScanning={false}
        movesRunning={true}
        rootMediaFolder={null}
        onChooseFolder={vi.fn()}
        onCancelScan={vi.fn()}
      />
    );

    expect(screen.getByRole("button", { name: "Choose Folder" })).toBeDisabled();
  });
});
