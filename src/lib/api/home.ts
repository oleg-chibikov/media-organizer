import type { DestinationSettings, HomeCandidate, HomeLocation } from "../types";
import { invokeCommand } from "./tauriRuntime";

/** Up to 10 cities whose name starts with, then contains, the query. */
export const searchHomeCities = (query: string) =>
  invokeCommand<HomeCandidate[]>("search_home_cities", { query });
/** Saves the picked home, or clears it with `null` so a large scan learns it again. */
export const setHomeLocation = (home: HomeLocation | null) =>
  invokeCommand<DestinationSettings>("set_home_location", { home });
