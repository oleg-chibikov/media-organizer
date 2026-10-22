import type { PlaceDetection } from "../types";
import { invokeCommand } from "./tauriRuntime";

/** The place folder of each photo and video of scan `scanId`, and whether any has GPS.
 * A `preview` runs while dates are still read, so it saves no learned home. */
export const detectFilePlaces = (scanId: number, fileIds: string[], preview: boolean) =>
  invokeCommand<PlaceDetection>("detect_file_places", { scanId, fileIds, preview });
