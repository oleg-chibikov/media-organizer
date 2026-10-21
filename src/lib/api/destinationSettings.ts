import type { DestinationSettings, EffectiveDestinationRoot } from "../types";
import { invokeCommand } from "./tauriRuntime";

export const getDestinationSettings = () =>
  invokeCommand<DestinationSettings>("get_destination_settings");
export const getEffectiveDestinationRoot = (selectedPath: string) =>
  invokeCommand<EffectiveDestinationRoot>("get_effective_destination_root", { selectedPath });
export const chooseRootMediaFolder = () =>
  invokeCommand<DestinationSettings | null>("choose_root_media_folder");
export const clearRootMediaFolder = () =>
  invokeCommand<DestinationSettings>("clear_root_media_folder");
export const setForceProcessedFolder = (forceProcessedFolder: boolean) =>
  invokeCommand<DestinationSettings>("set_force_processed_folder", {
    forceProcessedFolder
  });
