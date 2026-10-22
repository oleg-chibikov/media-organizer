type HomeSource = "learned" | "manual" | "device" | "time_zone";

export interface HomeLocation {
  name: string;
  country: string;
  latitude: number;
  longitude: number;
  source: HomeSource;
}

/** A city from the offline place table that can become home. */
export interface HomeCandidate {
  name: string;
  region: string | null;
  country: string;
  latitude: number;
  longitude: number;
}
