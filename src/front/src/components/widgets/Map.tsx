import * as react from 'react';
import * as maplibre from 'maplibre-gl';
import * as mapstyle from './MapStyle';
import type * as api from '../../api';
import tracker from '../../assets/tracker02.png';

interface MapProps {
  // center?: api.LngLat;
  // zoom?: number;
  // timeMs?: number,
  data: api.MapView,
  onMove?: (center: api.LngLat, zoom: number, bounds: api.BBox) => void;
  className?: string;
  id?: string;
}



export default function Map({ data: map, onMove, className, id }: MapProps) {
  const mapElementRef = react.useRef<HTMLDivElement>(null);
  const mapRef = react.useRef<maplibre.Map | null>(null);

  const notifyMove = () => {
    if (mapRef.current) {
      const { lng, lat } = mapRef.current.getCenter();
      const center = [lng, lat] as api.LngLat;
      const zoom = mapRef.current.getZoom();
      const bounds = fromMaplibreBounds(mapRef.current.getBounds());
      onMove && onMove(center, zoom, bounds);
    }
  };

  react.useEffect(() => {
    if (mapElementRef.current && !mapRef.current) {
      // Create map
      const style = mapstyle.getMapStyle({
        basemap: map,
      });
      if (import.meta.env.MODE !== 'production') {
        console.log("creating map widget, data:", map);
        console.log("style:", style);
      }
      mapRef.current = new maplibre.Map({
        container: mapElementRef.current,
        //style: 'https://demotiles.maplibre.org/style.json',
        style,
        center: map.center,
        zoom: map.zoom,
        bearing: map.bearingDeg,
      });
      notifyMove(); // force re-query with actual bounds
      mapRef.current.on('moveend', () => {
        notifyMove();
      });
      mapRef.current.on('load', () => {
        if (mapRef.current) {
          mapRef.current.loadImage(tracker, (err, image) => {
            if (err) throw err;
            if (!mapRef.current || !image) {
              return;
            }
            mapRef.current?.addImage('tracker', image);
          });
          }
      });
    }
    else if (mapRef.current) {
      // Update map
      const style = mapstyle.getMapStyle({
        center: map.center,
        bearing: map.bearingDeg,
        basemap: map,
      });
      if (import.meta.env.MODE !== 'production') {
        console.log("updating map widget, data:", map);
        console.log("style:", style);
      }
      mapRef.current.setStyle(style, { diff: true });
    }

    return () => {
      if (import.meta.env.MODE !== 'production') {
        if (mapRef.current) {
          console.log("Cleaning up map widget");
          mapRef.current.remove();
          mapRef.current = null;
        }
      }
    };
  }, [mapElementRef]);

  return <div ref={mapElementRef} className={className} id={id} />;
}


function fromMaplibreBounds(b: maplibre.LngLatBounds): api.BBox {
  const bbMinLng = Math.min(b.getEast(), b.getWest());
  const bbMaxLng = Math.max(b.getEast(), b.getWest());
  const bbMinLat = Math.min(b.getNorth(), b.getSouth());
  const bbMaxLat = Math.max(b.getNorth(), b.getSouth());
  return [bbMinLng, bbMinLat, bbMaxLng, bbMaxLat] as api.BBox;
}