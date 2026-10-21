import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { Toolbar } from "./Toolbar";

afterEach(cleanup);

const BASE_PROPS = {
  selectedFolder: "C:\\in",
  destinationRoot: "D:\\Media",
  destinationError: null,
  folderError: null,
  isScanning: false,
  movesRunning: false,
  onChooseFolder: vi.fn(),
  onCancelScan: vi.fn(),
  onDismissError: vi.fn()
};

describe("Toolbar", () => {
  it("shows where files come from and where they go", () => {
    render(<Toolbar {...BASE_PROPS} historyMenu={<button type="button">History</button>} />);

    expect(screen.getByText("From").nextElementSibling).toHaveTextContent("C:\\in");
    expect(screen.getByText("To").nextElementSibling).toHaveTextContent("D:\\Media");
    expect(screen.getByRole("button", { name: "History" })).toBeInTheDocument();
  });

  it("explains a destination the backend can't use", () => {
    render(<Toolbar {...BASE_PROPS} destinationRoot={null} destinationError="Folder is offline" />);

    expect(screen.getByText("Can't use the destination: Folder is offline")).toBeInTheDocument();
  });

  it("stops a scan and dismisses an error", () => {
    const onCancelScan = vi.fn();
    const onDismissError = vi.fn();
    render(
      <Toolbar
        {...BASE_PROPS}
        selectedFolder={null}
        folderError="Selected folder is inside a Processed path."
        isScanning={true}
        onCancelScan={onCancelScan}
        onDismissError={onDismissError}
      />
    );

    expect(screen.getByText("No folder selected")).toBeInTheDocument();
    expect(screen.getByRole("alert")).toHaveTextContent(
      "Selected folder is inside a Processed path."
    );
    fireEvent.click(screen.getByRole("button", { name: "Stop Scan" }));
    fireEvent.click(screen.getByRole("button", { name: "Dismiss" }));
    expect(onCancelScan).toHaveBeenCalledTimes(1);
    expect(onDismissError).toHaveBeenCalledTimes(1);
  });

  it("blocks a new folder while moves are running", () => {
    render(<Toolbar {...BASE_PROPS} movesRunning={true} />);

    expect(screen.getByRole("button", { name: "Choose Folder" })).toBeDisabled();
  });
});
