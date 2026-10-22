import type { ProcessingConfig } from "../types";
import { invokeCommand } from "./tauriRuntime";

export const healthCheck = () => invokeCommand<string>("health_check");
export const getProcessingConfig = () => invokeCommand<ProcessingConfig>("get_processing_config");
export const revealFileInExplorer = (sourcePath: string) =>
  invokeCommand<void>("reveal_file_in_explorer", { sourcePath });
export const openFile = (sourcePath: string) => invokeCommand<void>("open_file", { sourcePath });
