import * as react from "react";
import * as reactRouter from "react-router-dom";
import * as api from "./api";

/**
 * Navigation query that corresponds to browser location bar
 */
export interface NavQuery {
  timeMs?: number;
  center?: api.LngLat;
  zoom?: number;
}


/**
 * A React hook to use navigation query
 *
 * This is the "main" query that corresponds to browser location bar.
 *
 * @returns {Array} - an array of two elements. The first is the parsed
 *    query object. The second is the function to update the query. Calling
 *    this function will replace (not push) current navigation state.
 *
 * @example
 *   const [query, navigate] = useNavQuery();
 */
export function useNavQuery(): [NavQuery, (q: NavQuery) => void] {
  const [params, setParams] = reactRouter.useSearchParams();

  const navigate = (q: NavQuery) => {
    const nparams = new URLSearchParams(params);
    nparams.delete("ts");
    if (q.timeMs) {
      nparams.append("ts", new Date(q.timeMs).toISOString());
    }

    nparams.delete("c");
    if (q.center) {
      nparams.append(
        "c",
        `${q.center[0].toFixed(5)}_${q.center[1].toFixed(5)}`
      );
    }

    nparams.delete("z");
    if (q.zoom) {
      nparams.append("z", q.zoom.toFixed(3));
    }
    setParams(nparams, { replace: true });
  };

  let tsStr = params.get("ts");
  let ts = tsStr ? new Date(tsStr).getTime() : undefined;

  let zoomStr = params.get("z");
  let zoom = zoomStr === null ? undefined : parseFloat(zoomStr);

  let centerRaw = params.get("c")?.split("_").map(parseFloat);
  let center: api.LngLat | undefined;
  if (centerRaw?.length == 2 && !centerRaw.some(isNaN)) {
    const [lng, lat] = centerRaw;
    center = [lng, lat];
  } else {
    center = undefined;
  }
  const query = {
    ts,
    center,
    zoom,
  };
  return [query, navigate];
}

let theSession: api.Session|null = null;

/**
 * A React hook to use session
 */
export function useSession() {
  const [navQuery, _navigate] = useNavQuery();
  const session = react.useRef<api.Session | null>(null);

  if (!theSession) {
    // Build initial map query. Map container have not yet been
    // rendered, so we don't know it's size and thus the map bounds - but we'll
    // create some dummy bounds instead.
    console.log(`initial nav query:`, JSON.stringify(navQuery));
    const { center: c, timeMs, zoom } = navQuery;
    const apiQuery = {
      bounds: c ? ([c[0], c[1], c[0], c[1]] as api.BBox) : undefined,
      zoom,
      timeMs,
    };
    console.log(`initial api query:`, JSON.stringify(apiQuery));
    theSession = new api.Session(apiQuery, 5000);
  }

  const query = (q: api.Query) => {
    theSession!.query(q);
  };

  const dismissAlert = () => {
    theSession!.dismissAlert();
  };

  const subscribe = (listener: api.Listener) => {
    theSession!.on("update", listener);
    return () => theSession!.off("update", listener);
  };

  const snapshot = () => {
    return theSession!.state;
  };

  return {
    session: react.useSyncExternalStore(subscribe, snapshot),
    query,
    dismissAlert,
  };
}
