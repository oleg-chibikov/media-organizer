import type { FileKind, MetadataReadRequest, MetadataResult } from "../types";
import { invokeCommand } from "./tauriRuntime";

export const readMetadataForFile = (fileId: string, sourcePath: string, fileKind: FileKind) =>
  invokeCommand<MetadataResult>("read_metadata_for_file", {
    fileId,
    sourcePath,
    fileKind
  });
export const readMetadataForFiles = (requests: MetadataReadRequest[]) =>
  invokeCommand<MetadataResult[]>("read_metadata_for_files", { requests });
