import { useCallback, useEffect, useRef, useState } from "react";
import type { DestinationSettings } from "../lib/types";

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
  const [open, setOpen] = useState(false);
  const panelRef = useRef<HTMLDivElement>(null);
  const controlsDisabled = loading || pending || locked;
  const rootPath = settings?.root_media_folder ?? null;
  const warning = settings?.effective_root?.warning ?? settings?.warning ?? null;
  const effectiveMode =
    settings?.effective_root?.mode === "custom_root" ? "Custom root active" : "Processed active";

  const handleClickOutside = useCallback((event: MouseEvent) => {
    if (panelRef.current && !panelRef.current.contains(event.target as Node)) {
      setOpen(false);
    }
  }, []);

  useEffect(() => {
    if (open) {
      document.addEventListener("mousedown", handleClickOutside);
    }
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [open, handleClickOutside]);

  return (
    <div ref={panelRef} className="relative">
      <button
        className="rounded bg-slate-800 px-3 py-1.5 text-sm font-medium text-slate-100 hover:bg-slate-700"
        onClick={() => setOpen((previous) => !previous)}
        type="button"
      >
        Settings
      </button>
      {open ? (
        <section className="absolute right-0 z-30 mt-1 w-96 rounded border border-slate-700 bg-slate-900 p-3 shadow-lg">
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
            <button
              className="rounded bg-sky-600 px-2 py-1 text-xs font-medium text-white hover:bg-sky-500 disabled:cursor-not-allowed disabled:bg-slate-700"
              disabled={controlsDisabled}
              onClick={onChooseRoot}
              type="button"
            >
              Choose Root
            </button>
            <button
              className="rounded bg-slate-800 px-2 py-1 text-xs font-medium text-slate-100 hover:bg-slate-700 disabled:cursor-not-allowed disabled:bg-slate-700 disabled:text-slate-500"
              disabled={controlsDisabled || !rootPath}
              onClick={onClearRoot}
              type="button"
            >
              Clear Root
            </button>
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
      ) : null}
    </div>
  );
}
