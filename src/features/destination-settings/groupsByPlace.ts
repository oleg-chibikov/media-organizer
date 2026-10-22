import type { DestinationSettings } from "../../lib/types";

// Rename only wins: files stay in their folders, so no file gets a place folder.
export const groupsByPlace = (settings: DestinationSettings | null) =>
  (settings?.group_by_place ?? true) && !(settings?.rename_in_place ?? false);
