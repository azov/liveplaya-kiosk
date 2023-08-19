const ENDPOINT = "/api";

export type LngLat = [number, number];
export type BBox = [number, number, number, number];
export type Listener = () => void;

export interface Query {
  feature?: string,
  bounds?: BBox;
  zoom?: number;
}

export interface MapView {
  bearingDeg: number;
  center: LngLat;
  description: string,
  features: any[];
  refs: FeatureRef[];
  log: LogMessage[];
  name: string,
  time: object;
  // bounds: BBox;
  type: "FeatureCollection";
  zoom: number;
}

export interface TextView {
  name: string,
  description: string,
  timeStr: string;
  log: LogMessage[];
}

export interface BeaconRef {
  type: "beacon",
  name: string, 
  slug: string, 
  location: string, 
  lastseen: string,
}

export type FeatureRef = BeaconRef;

export interface LogMessage {
  id: number,
  level: "error"|"info"|"debug",
  text: string,
  time: string,
}


export type View = MapView;

export interface SessionState {
  isLoading: boolean;
  query: Query;
  view?: View;
  alert?: Alert;
}

type AlertLevel = "error" | "info" | "success";

export class Alert {
  constructor(public text: string, public level: AlertLevel = "info") {}
}

export class Session {
  public state: SessionState;
  private _listeners: Listener[] = [];
  private _loading: AbortController | null = null;

  constructor(query: Query = {}, refreshInterval = 5000) {
    if (import.meta.env.MODE !== "production") {
      console.log(`creating new session`);
    }

    this.state = {
      isLoading: true,
      query: query,
    };
    this._fetchAndNotify();
    if (refreshInterval) {
      setInterval(() => {
        this._fetchAndNotify();
      }, refreshInterval);
    }
  }

  query(query: Query) {
    this.state = {
      ...this.state,
      query,
    };
    this._fetchAndNotify();
  }

  dismissAlert() {
    const { alert, ...rest } = this.state;
    this.state = rest;
    this._notify("update");
  }

  on(evt: string, listener: Listener) {
    this._listeners = [...this._listeners, listener];
    return () => this.off(evt, listener);
  }

  off(_evt: string, listener: Listener) {
    const idx = this._listeners.indexOf(listener);
    if (idx >= 0) {
      this._listeners.splice(idx, 1);
    }
  }

  protected async _fetchAndNotify() {
    if (this._loading) {
      this._loading.abort();
    }
    this._loading = new AbortController();
    this.state = {
      ...this.state,
      isLoading: true,
    };
    this._notify("update");
    try {
      const q = this.state.query;
      const args = new URLSearchParams();
      // q.timeMs && args.append("time", new Date(q.timeMs).toISOString());
      q.zoom && args.append("zoom", q.zoom.toString());
      // q.bounds &&
      //   args.append("bounds", q.bounds.map((v) => v.toFixed(5)).join(","));
      const res = await fetch(`${ENDPOINT}/v0/?${args}`, {
        signal: this._loading.signal,
      });
      if (!res.ok) {
        throw new Error(`HTTP ${res.status}: ${await res.text()}`);
      }
      const view = (await res.json()) as unknown as View;
      this.state = {
        ...this.state,
        isLoading: false,
        view,
      };
      this._notify("update");
    } catch (e) {
      // In case we're aborted - do nothing as aborter set a new
      // status and isLoading flag
      if (!(e instanceof Error) || e.name != "AbortError") {
        const msg = `Failed to fetch data: ${e}`;
        console.error(msg);
        this.state = {
          ...this.state,
          isLoading: false,
          alert: new Alert(msg, "error"),
        };
        this._notify("update");
      }
    }
  }

  protected _notify(_evt: string) {
    for (let listener of this._listeners) {
      listener();
    }
  }
}
