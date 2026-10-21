import type { MetadataReadRequest, MetadataResult } from "../types";
import { invokeCommand } from "./tauriRuntime";

export const readMetadataForFiles = (requests: MetadataReadRequest[]) =>
  invokeCommand<MetadataResult[]>("read_metadata_for_files", { requests });
