import type { PlaceDetection } from "../types";
import { invokeCommand } from "./tauriRuntime";

/** The place folder of each photo and video of scan `scanId`, and whether any has GPS. */
export const detectFilePlaces = (scanId: number, fileIds: string[]) =>
  invokeCommand<PlaceDetection>("detect_file_places", { scanId, fileIds });
