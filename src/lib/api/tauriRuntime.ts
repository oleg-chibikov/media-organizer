import { invoke } from "@tauri-apps/api/core";

const TAURI_RUNTIME_ERROR =
  "Tauri runtime is unavailable. Start the app with Tauri (for example `mise x -- pnpm tauri dev`), not plain `vite`/web dev mode.";

export function ensureTauriRuntime(): void {
  const tauriInternals = (window as Window & { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__;

  if (!tauriInternals) {
    throw new Error(TAURI_RUNTIME_ERROR);
  }
}

export function invokeCommand<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  ensureTauriRuntime();
  return invoke<T>(command, args);
}
