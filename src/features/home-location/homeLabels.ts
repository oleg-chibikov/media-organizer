import type { HomeCandidate, HomeLocation } from "../../lib/types";

export function homeStatus(home: HomeLocation | null): string {
  if (!home) {
    return "Home: not set";
  }
  return `Home: ${home.name}, ${home.country} (${home.source})`;
}

export function candidateLabel({ name, region, country }: HomeCandidate): string {
  return [name, region, country].filter(Boolean).join(", ");
}

export function homeFromCandidate(candidate: HomeCandidate): HomeLocation {
  const { name, country, latitude, longitude } = candidate;
  return { name, country, latitude, longitude, source: "manual" };
}
