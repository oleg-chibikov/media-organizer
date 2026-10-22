import type { HomeCandidate, HomeLocation, HomeSource } from "../../lib/types";

export function homeStatus(home: HomeLocation | null): string {
  if (!home) {
    return "Home: not set";
  }
  return `Home: ${home.name}, ${home.country}`;
}

const SOURCE_LABELS: Record<HomeSource, string> = {
  manual: "picked by you",
  device: "from your location",
  learned: "learned from photos",
  time_zone: "from time zone"
};

export function homeSourceLabel(source: HomeSource): string {
  return SOURCE_LABELS[source];
}

export function candidateLabel({ name, region, country }: HomeCandidate): string {
  return [name, region, country].filter(Boolean).join(", ");
}

export function homeFromCandidate(candidate: HomeCandidate): HomeLocation {
  const { name, country, latitude, longitude } = candidate;
  return { name, country, latitude, longitude, source: "manual" };
}
