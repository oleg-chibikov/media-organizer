import { useCallback, useState } from "react";
import type { FilePlace } from "../../lib/types";
import type { PlaceMapTarget } from "../place-map";
import type { FileRowState } from "./fileRowState";

export function usePlaceMapTarget(filePlaces: Record<string, FilePlace>) {
  const [mapTarget, setMapTarget] = useState<PlaceMapTarget | null>(null);
  const openMap = useCallback(
    ({ record, metadata }: FileRowState) => {
      if (metadata?.gps) {
        const found = filePlaces[record.id];
        const label = found ? [found.place, found.country].filter(Boolean).join(", ") : "";
        setMapTarget({ point: metadata.gps, label });
      }
    },
    [filePlaces]
  );
  const closeMap = useCallback(() => setMapTarget(null), []);
  return { mapTarget, openMap, closeMap };
}
