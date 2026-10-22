import { useEffect, useRef } from "react";
import { circleMarker, map as createMap, tileLayer } from "leaflet";
import "leaflet/dist/leaflet.css";
import type { GpsPoint } from "../../lib/types";

const STREET_ZOOM = 15;

interface LeafletMapProps {
  point: GpsPoint;
  label: string;
}

// A circle marker avoids Leaflet's default icon images, which Vite doesn't bundle.
export function LeafletMap({ point, label }: LeafletMapProps) {
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const container = containerRef.current;
    if (!container) {
      return;
    }
    const center: [number, number] = [point.latitude, point.longitude];
    const leafletMap = createMap(container).setView(center, STREET_ZOOM);
    tileLayer("https://tile.openstreetmap.org/{z}/{x}/{y}.png", {
      maxZoom: 19,
      attribution: "&copy; OpenStreetMap contributors"
    }).addTo(leafletMap);
    circleMarker(center, { radius: 9, color: "#0284c7", fillOpacity: 0.6 })
      .bindTooltip(label)
      .addTo(leafletMap);
    return () => {
      leafletMap.remove();
    };
  }, [point.latitude, point.longitude, label]);

  return (
    <div
      ref={containerRef}
      role="img"
      aria-label={`Map of ${label}`}
      className="h-96 w-full rounded border border-slate-700"
    />
  );
}
