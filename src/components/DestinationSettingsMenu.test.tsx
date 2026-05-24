import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { DestinationSettingsMenu } from "./DestinationSettingsMenu";
import type { DestinationSettings } from "../lib/types";

const SETTINGS: DestinationSettings = {
  version: 1,
  root_media_folder: "C:\\Media",
  force_processed_folder: false,
  updated_at: "2026-05-24T00:00:00Z",
  effective_root: {
    mode: "custom_root",
    root_path: "C:\\Media",
    custom_root_configured: true,
    force_processed_folder: false,
    warning: null
  }
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

    expect(screen.getByText("Destination Settings")).toBeInTheDocument();
    expect(screen.getByText("Custom root active")).toBeInTheDocument();
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
    fireEvent.click(screen.getByRole("button", { name: "Choose Root" }));
    fireEvent.click(screen.getByRole("button", { name: "Clear Root" }));
    fireEvent.click(screen.getByRole("checkbox", { name: "Force Processed folder" }));

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
      screen.getByText("Settings are locked while move execution is active.")
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Choose Root" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Clear Root" })).toBeDisabled();
    expect(screen.getByRole("checkbox", { name: "Force Processed folder" })).toBeDisabled();
  });
});
