import { lazy, Suspense } from "react";
import { Button } from "../../components/design-system/Button";
import { Dialog } from "../../components/design-system/Dialog";
import type { GpsPoint } from "../../lib/types";

// Leaflet is only needed once the dialog opens, so it stays out of the startup bundle.
const LeafletMap = lazy(() =>
  import("./LeafletMap").then(({ LeafletMap }) => ({ default: LeafletMap }))
);

export interface PlaceMapTarget {
  point: GpsPoint;
  label: string;
}

interface PlaceMapDialogProps {
  target: PlaceMapTarget;
  onClose: () => void;
}

function formatCoordinates({ latitude, longitude }: GpsPoint) {
  return `${latitude.toFixed(5)}, ${longitude.toFixed(5)}`;
}

export function PlaceMapDialog({ target, onClose }: PlaceMapDialogProps) {
  const coordinates = formatCoordinates(target.point);

  return (
    <Dialog
      title={target.label || coordinates}
      wide
      onCancel={onClose}
      actions={
        <Button className="px-3 py-1.5 text-sm" onClick={onClose}>
          Close
        </Button>
      }
    >
      <Suspense
        fallback={<div className="h-96 w-full rounded border border-slate-700" aria-busy="true" />}
      >
        <LeafletMap point={target.point} label={target.label || coordinates} />
      </Suspense>
      <p className="mt-2 font-mono text-xs text-slate-400">{coordinates}</p>
    </Dialog>
  );
}
