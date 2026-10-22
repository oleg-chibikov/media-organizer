import type { HomeLocation } from "./home";

type DestinationRootMode = "custom_root" | "processed_folder" | "in_place";

export interface EffectiveDestinationRoot {
  mode: DestinationRootMode;
  root_path: string;
  custom_root_configured: boolean;
  force_processed_folder: boolean;
  warning: string | null;
}

export interface DestinationSettings {
  version: number;
  root_media_folder: string | null;
  force_processed_folder: boolean;
  rename_in_place: boolean;
  home: HomeLocation | null;
  updated_at: string;
}
