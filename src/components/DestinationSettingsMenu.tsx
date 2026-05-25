import type { DestinationSettings } from "../lib/types";
import { Button } from "./Button";
import { Pane } from "./Pane";

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
  const effectiveMode =
    settings?.force_processed_folder || !rootPath
      ? "Processed active"
      : settings?.effective_root?.mode === "processed_folder"
        ? "Processed active"
        : "Custom root active";

  return (
    <Pane
      align="right"
      panelClassName="z-30 w-96 p-3"
      renderTrigger={({ toggle }) => (
        <Button variant="secondary" aria-label="Settings" className="p-1.5" onClick={toggle}>
          <svg aria-hidden="true" className="h-5 w-5" viewBox="0 0 24 24" fill="none">
            <path
              d="M12 8.5a3.5 3.5 0 1 0 0 7 3.5 3.5 0 0 0 0-7Zm9 3.5-.9-.5a8.6 8.6 0 0 0-.3-1.2l.6-.8a1 1 0 0 0-.1-1.3l-1.5-1.5a1 1 0 0 0-1.3-.1l-.8.6c-.4-.1-.8-.3-1.2-.3L15 3a1 1 0 0 0-1-.8h-2a1 1 0 0 0-1 .8l-.2 1.1c-.4.1-.8.2-1.2.3l-.8-.6a1 1 0 0 0-1.3.1L6 5.4a1 1 0 0 0-.1 1.3l.6.8c-.1.4-.3.8-.3 1.2L5 9a1 1 0 0 0-.8 1v2a1 1 0 0 0 .8 1l1.1.2c.1.4.2.8.3 1.2l-.6.8a1 1 0 0 0 .1 1.3l1.5 1.5a1 1 0 0 0 1.3.1l.8-.6c.4.1.8.3 1.2.3l.2 1.1a1 1 0 0 0 1 .8h2a1 1 0 0 0 1-.8l.2-1.1c.4-.1.8-.2 1.2-.3l.8.6a1 1 0 0 0 1.3-.1l1.5-1.5a1 1 0 0 0 .1-1.3l-.6-.8c.1-.4.3-.8.3-1.2l1.1-.2a1 1 0 0 0 .8-1v-2a1 1 0 0 0-.8-1Z"
              stroke="currentColor"
              strokeWidth="1.4"
              strokeLinecap="round"
              strokeLinejoin="round"
            />
          </svg>
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
