import type { FileRecord, MetadataResult, PlaceSuggestion } from "../types";
import { invokeCommand } from "./tauriRuntime";

/** Trip and outing folders to suggest for the scanned files. */
export const detectPlaceSuggestions = (files: FileRecord[], metadataResults: MetadataResult[]) =>
  invokeCommand<PlaceSuggestion[]>("detect_place_suggestions", { files, metadataResults });
