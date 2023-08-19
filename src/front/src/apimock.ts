import {
  LngLat,
  BBox,
  Listener,
  Query,
  MapView,
  SessionState,
  Session as BaseSession,
} from "./api";
export type { LngLat, BBox, Listener, Query, MapView as MapSnapshot, SessionState };

export class Session extends BaseSession {
  protected async _fetchAndNotify() {
    console.assert(this.state.isLoading);
    setTimeout(() => {
      const ts = this.state.query.timeMs;
      const b = this.state.query.bounds||[0,0,0,0] as BBox;
      const center = [(b[0]+b[2])/2, (b[1]+b[3])/2] as LngLat;
      const snapshot = {
        bearingDeg: 45,
        center,
        bounds: b,
        features: [],
        timeStr: (ts ? new Date(ts) : new Date()).toISOString(),
        type: "FeatureCollection" as "FeatureCollection",
        zoom: this.state.query.zoom || 5,        
      }

      this.state = {
        ...this.state,
        isLoading: false,
        view: snapshot,
      };
      this._notify("update");
    }, 1000);
  }
}
