import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { DestinationSettingsMenu } from "./DestinationSettingsMenu";
import type { DestinationSettings } from "../../lib/types";

const SETTINGS: DestinationSettings = {
  version: 1,
  root_media_folder: "C:\\Media",
  force_processed_folder: false,
  rename_in_place: false,
  group_by_place: true,
  home: null,
  updated_at: "2026-05-24T00:00:00Z"
};

describe("DestinationSettingsMenu", () => {
  afterEach(() => {
    cleanup();
  });

  it("renders root folder status", () => {
    render(
      <DestinationSettingsMenu
        settings={SETTINGS}
        loading={false}
        pending={false}
        locked={false}
        error={null}
        onChooseRoot={vi.fn()}
        onClearRoot={vi.fn()}
        onSetForceProcessed={vi.fn()}
      />
    );

    fireEvent.click(screen.getByRole("button", { name: "Settings" }));

    expect(screen.getByText("Where files go")).toBeInTheDocument();
    expect(screen.getByText("C:\\Media")).toBeInTheDocument();
  });

  it("calls interaction handlers", () => {
    const onChooseRoot = vi.fn();
    const onClearRoot = vi.fn();
    const onSetForceProcessed = vi.fn();

    render(
      <DestinationSettingsMenu
        settings={SETTINGS}
        loading={false}
        pending={false}
        locked={false}
        error={null}
        onChooseRoot={onChooseRoot}
        onClearRoot={onClearRoot}
        onSetForceProcessed={onSetForceProcessed}
      />
    );

    fireEvent.click(screen.getByRole("button", { name: "Settings" }));
    fireEvent.click(screen.getByRole("button", { name: "Choose Library Folder" }));
    fireEvent.click(screen.getByRole("button", { name: "Clear" }));
    fireEvent.click(
      screen.getByRole("checkbox", { name: "Use a Processed folder inside the scanned folder" })
    );

    expect(onChooseRoot).toHaveBeenCalledTimes(1);
    expect(onClearRoot).toHaveBeenCalledTimes(1);
    expect(onSetForceProcessed).toHaveBeenCalledWith(true);
  });

  it("disables settings changes when locked", () => {
    render(
      <DestinationSettingsMenu
        settings={SETTINGS}
        loading={false}
        pending={false}
        locked={true}
        error={null}
        onChooseRoot={vi.fn()}
        onClearRoot={vi.fn()}
        onSetForceProcessed={vi.fn()}
      />
    );

    fireEvent.click(screen.getByRole("button", { name: "Settings" }));

    expect(
      screen.getByText("Locked after you start moving files. Choose a folder again to change it.")
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Choose Library Folder" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Clear" })).toBeDisabled();
    expect(
      screen.getByRole("checkbox", { name: "Use a Processed folder inside the scanned folder" })
    ).toBeDisabled();
  });

  it("refreshes settings only as it opens", () => {
    const onOpen = vi.fn();
    render(
      <DestinationSettingsMenu
        settings={SETTINGS}
        loading={false}
        pending={false}
        locked={false}
        error={null}
        onChooseRoot={vi.fn()}
        onClearRoot={vi.fn()}
        onSetForceProcessed={vi.fn()}
        onOpen={onOpen}
      />
    );

    fireEvent.click(screen.getByRole("button", { name: "Settings" }));
    fireEvent.click(screen.getByRole("button", { name: "Settings" }));

    expect(onOpen).toHaveBeenCalledTimes(1);
  });

  it("shows the home section while open", () => {
    render(
      <DestinationSettingsMenu
        settings={SETTINGS}
        loading={false}
        pending={false}
        locked={false}
        error={null}
        onChooseRoot={vi.fn()}
        onClearRoot={vi.fn()}
        onSetForceProcessed={vi.fn()}
        homeSection={<p>Home: not set</p>}
      />
    );

    fireEvent.click(screen.getByRole("button", { name: "Settings" }));

    expect(screen.getByText("Home: not set")).toBeInTheDocument();
  });
});
