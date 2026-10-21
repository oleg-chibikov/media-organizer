import { Button } from "../../components/design-system/Button";
import { Pane } from "../../components/design-system/Pane";
import type { DestinationSettings } from "../../lib/types";
import { effectiveDestinationMode } from "./effectiveDestinationMode";
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
}

export function DestinationSettingsMenu({
  settings,
  loading,
  pending,
  locked,
  error,
  onChooseRoot,
  onClearRoot,
  onSetForceProcessed
}: DestinationSettingsMenuProps) {
  const controlsDisabled = loading || pending || locked;
  const rootPath = settings?.root_media_folder ?? null;
  const warning = settings?.effective_root?.warning ?? settings?.warning ?? null;
  const effectiveMode = effectiveDestinationMode(settings);

  return (
    <Pane
      align="right"
      panelClassName="z-30 w-96 p-3"
      renderTrigger={({ toggle }) => (
        <Button variant="secondary" aria-label="Settings" className="p-1.5" onClick={toggle}>
          <SettingsIcon />
        </Button>
      )}
    >
      <section>
        <h2 className="text-sm font-semibold text-slate-100">Destination Settings</h2>
        <p className="mt-1 text-xs text-slate-400">{effectiveMode}</p>
        <div className="mt-3 rounded border border-slate-800 bg-slate-950 p-2">
          <p className="text-[11px] uppercase tracking-wide text-slate-500">Root Media Folder</p>
          {rootPath ? (
            <p className="mt-1 break-all font-mono text-xs text-slate-200">{rootPath}</p>
          ) : (
            <p className="mt-1 text-xs text-slate-500">Not set</p>
          )}
        </div>
        {warning ? <p className="mt-2 text-xs text-amber-300">{warning}</p> : null}
        {error ? <p className="mt-2 text-xs text-rose-400">{error}</p> : null}
        {locked ? (
          <p className="mt-2 text-xs text-slate-400">
            Settings are locked while move execution is active.
          </p>
        ) : null}

        <div className="mt-3 flex gap-2">
          <Button
            variant="primary"
            className="px-2 py-1 text-xs"
            disabled={controlsDisabled}
            onClick={onChooseRoot}
          >
            Choose Root
          </Button>
          <Button
            variant="secondary"
            className="px-2 py-1 text-xs disabled:text-slate-500"
            disabled={controlsDisabled || !rootPath}
            onClick={onClearRoot}
          >
            Clear Root
          </Button>
        </div>

        <label className="mt-3 flex items-center gap-2 text-xs text-slate-200">
          <input
            checked={settings?.force_processed_folder ?? false}
            disabled={controlsDisabled}
            onChange={(event) => onSetForceProcessed(event.target.checked)}
            type="checkbox"
          />
          Force Processed folder
        </label>
      </section>
    </Pane>
  );
}
