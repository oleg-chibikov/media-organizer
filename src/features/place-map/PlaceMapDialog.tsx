import { Button } from "../../components/design-system/Button";
import { Dialog } from "../../components/design-system/Dialog";
import type { GpsPoint } from "../../lib/types";
import { LeafletMap } from "./LeafletMap";

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
      <LeafletMap point={target.point} label={target.label || coordinates} />
      <p className="mt-2 font-mono text-xs text-slate-400">{coordinates}</p>
    </Dialog>
  );
}
