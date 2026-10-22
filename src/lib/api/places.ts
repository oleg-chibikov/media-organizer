import type { PlaceDetection } from "../types";
import { invokeCommand } from "./tauriRuntime";

/** Trip and outing folders to suggest for the scanned files, a place label per geotagged
 * file, and why nothing was found. A suggestion that continues a folder an earlier run
 * filled under the same destination carries `continues`. */
export const detectPlaceSuggestions = (selectedPath: string, scanId: number, fileIds: string[]) =>
  invokeCommand<PlaceDetection>("detect_place_suggestions", { selectedPath, scanId, fileIds });
