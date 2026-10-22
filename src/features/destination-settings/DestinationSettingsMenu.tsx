import type { ReactNode } from "react";
import { Button } from "../../components/design-system/Button";
import { Pane } from "../../components/design-system/Pane";
import type { DestinationSettings } from "../../lib/types";
import { RootFolderStatus } from "./RootFolderStatus";
import { SettingsIcon } from "./SettingsIcon";

interface DestinationSettingsMenuProps {
  settings: DestinationSettings | null;
  loading: boolean;
  pending: boolean;
  locked: boolean;
  error: string | null;
  onChooseRoot: () => void;
  onClearRoot: () => void;
  onSetForceProcessed: (nextValue: boolean) => void;
  // Called as the menu opens, so it shows settings the backend changed on its own.
  onOpen?: () => void;
  homeSection?: ReactNode;
}

export function DestinationSettingsMenu({
  settings,
  loading,
  pending,
  locked,
  error,
  onChooseRoot,
  onClearRoot,
  onSetForceProcessed,
  onOpen,
  homeSection
}: DestinationSettingsMenuProps) {
  const controlsDisabled = loading || pending || locked;
  const rootPath = settings?.root_media_folder ?? null;

  return (
    <Pane
      align="right"
      panelClassName="z-30 w-96 p-3"
      renderTrigger={({ open, toggle }) => (
        <Button
          variant="secondary"
          aria-expanded={open}
          aria-label="Settings"
          className="p-1.5"
          onClick={() => {
            if (!open) {
              onOpen?.();
            }
            toggle();
          }}
        >
          <SettingsIcon />
        </Button>
      )}
    >
      <section>
        <h2 className="text-sm font-semibold text-slate-100">Where files go</h2>
        <p className="mt-1 text-xs text-slate-400">
          Files go into a folder for each year, month and day.
        </p>
        <RootFolderStatus rootPath={rootPath} />
        {error ? (
          <p role="alert" className="mt-2 text-xs text-rose-400">
            {error}
          </p>
        ) : null}
        {locked ? (
          <p className="mt-2 text-xs text-slate-400">
            Locked after you start moving files. Choose a folder again to change it.
          </p>
        ) : null}

        <div className="mt-3 flex gap-2">
          <Button
            variant="primary"
            className="px-2 py-1 text-xs"
            disabled={controlsDisabled}
            onClick={onChooseRoot}
          >
            Choose Library Folder
          </Button>
          <Button
            variant="secondary"
            className="px-2 py-1 text-xs disabled:text-slate-500"
            disabled={controlsDisabled || !rootPath}
            onClick={onClearRoot}
          >
            Clear
          </Button>
        </div>

        <label className="mt-3 flex items-start gap-2 text-xs text-slate-200">
          <input
            aria-describedby="force-processed-hint"
            checked={settings?.force_processed_folder ?? false}
            className="mt-0.5"
            disabled={controlsDisabled}
            onChange={(event) => onSetForceProcessed(event.target.checked)}
            type="checkbox"
          />
          Use a Processed folder inside the scanned folder
        </label>
        <p id="force-processed-hint" className="ml-5 mt-0.5 text-xs text-slate-400">
          The library folder is kept but not used while this is on.
        </p>
      </section>
      {homeSection}
    </Pane>
  );
}
