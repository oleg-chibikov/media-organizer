export type DestinationRootMode = "custom_root" | "processed_folder";

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
  updated_at: string;
}
