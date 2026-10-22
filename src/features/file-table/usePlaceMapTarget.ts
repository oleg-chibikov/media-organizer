import { useCallback, useState } from "react";
import type { PlaceMapTarget } from "../place-map";
import type { FileRowState } from "./fileRowState";

export function usePlaceMapTarget(filePlaces: Record<string, string>) {
  const [mapTarget, setMapTarget] = useState<PlaceMapTarget | null>(null);
  const openMap = useCallback(
    ({ record, metadata }: FileRowState) => {
      if (metadata?.gps) {
        setMapTarget({ point: metadata.gps, label: filePlaces[record.id] ?? "" });
      }
    },
    [filePlaces]
  );
  const closeMap = useCallback(() => setMapTarget(null), []);
  return { mapTarget, openMap, closeMap };
}
