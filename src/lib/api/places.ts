import type { FileRecord, MetadataResult, PlaceSuggestion } from "../types";
import { invokeCommand } from "./tauriRuntime";

/** Trip and outing folders to suggest for the scanned files. A suggestion that continues
 * a folder an earlier run filled under the same destination carries `continues`. */
export const detectPlaceSuggestions = (
  selectedPath: string,
  files: FileRecord[],
  metadataResults: MetadataResult[]
) =>
  invokeCommand<PlaceSuggestion[]>("detect_place_suggestions", {
    selectedPath,
    files,
    metadataResults
  });
