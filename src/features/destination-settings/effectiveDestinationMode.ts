import type { DestinationSettings } from "../../lib/types";

export function effectiveDestinationMode(settings: DestinationSettings | null): string {
  const usesProcessedFolder =
    settings?.force_processed_folder ||
    !settings?.root_media_folder ||
    settings.effective_root?.mode === "processed_folder";
  return usesProcessedFolder ? "Processed active" : "Custom root active";
}
