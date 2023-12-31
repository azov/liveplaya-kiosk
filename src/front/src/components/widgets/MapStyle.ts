import * as maplibre from "maplibre-gl";

export interface Style extends maplibre.StyleSpecification {
  metadata: {
    padding?: number;
    maxbounds?: number[];
  };
}

export function getMapStyle(
  options: {
    center?: number[];
    bearing?: number;
    maxbounds?: number[];
    basemap?: object;
    rastermap?: boolean;
  } = {}
) {
  let { basemap, rastermap } = options;
  const COLORS = {
    bgcolor01: "#121212",
    bgcolor02: "#535353",
    fgcolor01: "#b3b3b3",
    fgcolor02: "#1db954",
    fgcolor03: "#ff5555",

    background: "#fbfaf7",
    bgfeature: "#d1be9d",
    //bgaccent: "#cccccc",

    fgfeature: "#b9a37e",
    fgaccent1: "#3b727c",
    fgaccent2: "#82a775",
    fgaccent3: "#b05f66",
    fgcontrast: "#64513b",

    // ---
    fgmuted: "#ff0000", //"#cccccc",
    bgregular: "#ffffff",

    inactive: "#cccccc",
    common1: "#ff1493", //"#f032e6",
    common2: "#ff8c00", //"#bfef45",
    common3: "#00ff7f", //"#dcbeff",
    highlight: "#ffff00",
    outline: "#000",
    cursor: "#ffff00",
    activeFeature: "#17fcf5",
  };

  if (!basemap) {
    const emptyCollection = {
      type: "FeatureCollection",
      features: [],
    };
    if (!basemap) basemap = emptyCollection;
  }

  const style: Style = {
    version: 8,
    metadata: {
      padding: 60,
    },
    glyphs: "/fonts/{fontstack}/{range}.pbf",
    sources: {
      osmtiles: {
        type: "raster",
        tiles: ["https://a.tile.openstreetmap.org/{z}/{x}/{y}.png"],
        tileSize: 256,
        attribution: "&copy; OpenStreetMap Contributors",
        maxzoom: 19,
      },
      mapdata: {
        type: "geojson",
        data: basemap,
        generateId: true,
      },
    },
    layers: [
      {
        id: "background",
        type: "background",
        paint: {
          "background-color": COLORS.bgcolor01,
        },
      },
      {
        id: "rastermap",
        type: "raster",
        source: "osmtiles",
        metadata: {
          visibilityFlag: "showMap",
          visibilityFlagInverted: false,
        },
      },
      {
        id: "camping",
        source: "mapdata",
        type: "fill",
        filter: ["==", ["get", "brc"], "camping"],
        metadata: {},
        paint: {
          "fill-color": COLORS.bgcolor02,
          "fill-opacity": 0.3,
        },
      },
      {
        id: "bmorg-outlines",
        source: "mapdata",
        type: "line",
        filter: ["==", ["get", "liveplaya"], "bmorg-street-outlines"],
        paint: {
          "line-width": 1,
          "line-color": COLORS.fgcolor02,
          "line-opacity": 0.7,
        },
      },
      // {
      //   id: "streets",
      //   source: "mapdata",
      //   type: "line",
      //   filter: ["==", ["get", "brc"], "street"],
      //   metadata: {
      //   },
      //   paint: {
      //     "line-width": 10,
      //     "line-color": COLORS.background,
      //     "line-opacity": 1,
      //   },
      // },
      {
        id: "streets",
        source: "mapdata",
        type: "fill",
        filter: ["==", ["get", "brc"], "street"],
        metadata: {},
        paint: {
          "fill-color": COLORS.bgcolor01,
        },
      },
      {
        id: "perimeter",
        source: "mapdata",
        type: "line",
        filter: ["==", ["get", "brc"], "perimeter"],
        paint: {
          "line-width": 1,
          "line-color": COLORS.fgcolor02,
          "line-opacity": 1,
          "line-dasharray": [5, 5],
        },
      },
      {
        id: "tstreetend-labels",
        source: "mapdata",
        minzoom: 12.5,
        type: "symbol",
        filter: ["==", ["get", "liveplaya"], "radialend"],
        metadata: {},
        layout: {
          "text-field": ["get", "name"],
          "text-offset": [1.9, 0],
          "text-anchor": "left",
          "text-rotate": ["+", -90, ["get", "dir"]],
          "text-rotation-alignment": "viewport",
          // "text-radial-offset": -10,
          "text-allow-overlap": true,
          "text-size": ["interpolate", ["linear"], ["zoom"], 12.5, 8, 16, 16],
        },
        paint: {
          "text-color": COLORS.fgcolor02,
          "text-opacity": 1,
          "text-halo-width": 1,
          "text-halo-color": COLORS.bgcolor01,
        },
      },
      {
        id: "cstreetstart-labels",
        source: "mapdata",
        minzoom: 12.5,
        type: "symbol",
        filter: ["==", ["get", "cstreet"], "start"],
        metadata: {},
        layout: {
          "text-field": ["get", "name"],
          "text-offset": [-1, 0],
          "text-anchor": "right",
          "text-rotate": ["get", "tandg"],
          "text-rotation-alignment": "map",
          "text-allow-overlap": true,
          "text-size": ["interpolate", ["linear"], ["zoom"], 12.5, 8, 16, 16],
        },
        paint: {
          "text-color": COLORS.fgcolor02,
          "text-opacity": 1,
          "text-halo-width": 1,
          "text-halo-color": COLORS.bgcolor01,
        },
      },
      {
        id: "cstreetend-labels",
        source: "mapdata",
        minzoom: 12.5,
        // maxzoom: 14.5,
        type: "symbol",
        filter: ["==", ["get", "cstreet"], "end"],
        metadata: {},
        layout: {
          "text-field": ["get", "name"],
          "text-offset": [1, 0],
          "text-anchor": "left",
          "text-rotate": ["get", "tandg"],
          "text-rotation-alignment": "map",
          "text-allow-overlap": true,
          "text-size": ["interpolate", ["linear"], ["zoom"], 12.5, 8, 16, 16],
        },
        paint: {
          "text-color": COLORS.fgcolor02,
          "text-opacity": 1,
          "text-halo-width": 1,
          "text-halo-color": COLORS.bgcolor01,
        },
      },
      {
        id: "road-labels",
        source: "mapdata",
        type: "symbol",
        filter: [
          "any",
          ["==", ["get", "liveplaya"], "cstreet"],
          ["==", ["get", "liveplaya"], "streetcenter"],
        ],
        minzoom: 14.5,
        metadata: {
          visibilityFlag: "hideEvt",
          visibilityFlagInverted: true,
        },
        layout: {
          "symbol-placement": "line",
          "symbol-spacing": 400,
          "text-field": ["get", "name"],
          //'text-font': ['Open Sans Semibold', 'Arial Unicode MS Bold'],
          //"text-offset": [0, -0.05],
          "text-anchor": "center",
          //"text-keep-upright": false,
          //"text-allow-overlap": false,
          //'text-ignore-placement': true,
          "text-size": ["interpolate", ["linear"], ["zoom"], 12.5, 8, 16, 16],
        },
        paint: {
          "text-color": COLORS.fgcolor02,
          "text-opacity": 1,
          "text-halo-width": 1.5,
          "text-halo-color": COLORS.bgcolor01,
        },
      },
      // {
      //   id: "status-labels",
      //   source: "mapdata", // rtfeatures
      //   //minzoom: 14,
      //   type: "symbol",
      //   filter: ["has", "status"],
      //   metadata: {},
      //   layout: {
      //     "text-field": ["get", "status"],
      //     "text-optional": false,
      //     "text-offset": [0, 3],
      //     "text-anchor": "center",
      //     "text-allow-overlap": true,
      //     "text-size": 10,
      //   },
      //   paint: {
      //     "text-color": COLORS.fgaccent1,
      //     "text-opacity": 1,
      //     "text-halo-width": 1.5,
      //     "text-halo-color": COLORS.background,
      //   },
      // },
      // {
      //   id: "location-labels",
      //   source: "mapdata", // rtfeatures
      //   // minzoom: 14,
      //   type: "symbol",
      //   filter: ["has", "location"],
      //   metadata: {
      //     visibilityFlag: "hideEvt",
      //     visibilityFlagInverted: true,
      //   },
      //   layout: {
      //     "text-field": ["get", "location"],
      //     "text-offset": [0, 3],
      //     'text-anchor': 'center',
      //     "text-allow-overlap": false,
      //     // "text-size": ["match", ["get", "size"], "small", 12, "large", 16, 14],
      //     "text-size": 11,
      //   },
      //   paint: {
      //     "text-color": COLORS.fgcolor03,
      //     "text-opacity": 1,
      //     "text-halo-width": 1.5,
      //     "text-halo-color": COLORS.bgcolor01,
      //   },
      // },
      {
        id: "pois",
        source: "mapdata", // rtfeatures
        // minzoom: 14,
        type: "symbol",
        filter: ["==", ["get", "poi"], "beacon"],
        metadata: {
          visibilityFlag: "hideEvt",
          visibilityFlagInverted: true,
        },
        layout: {
          "symbol-sort-key": ["get", "priority"],
          "icon-image": "tracker",
          "icon-size": [
            "match",
            ["get", "size"],
            "small",
            0.3,
            "large",
            0.7,
            0.5,
          ],
          "icon-rotate": ["get", "headingDeg"],
          //"icon-overlap": "always",
          //"symbol-sort-key": ["get", "order"],
          "text-field": ["concat", ["get", "name"], "\n", ["get", "location"]],
          "text-optional": true,
          //'text-font': ['Open Sans Regular'],
          "text-offset": [0, 2],
          "text-anchor": "center",
          // "text-variable-anchor": ["top", "bottom", "top-left", "bottom-right"],
          "text-allow-overlap": false,
          //'text-ignore-placement': true,
          "text-size": ["match", ["get", "size"], "small", 12, "large", 16, 14],
        },
        paint: {
          "icon-color": COLORS.fgcolor02,
          "icon-halo-color": COLORS.bgcolor01,
          "icon-halo-width": 1,
          "icon-opacity": ["case", ["get", "recent"], 1, 0.5],
          "text-color": '#ff5555', //COLORS.fgcolor02,
          "text-opacity": ["case", ["get", "recent"], 1, 0.5],
          "text-halo-width": 1.5,
          "text-halo-color": COLORS.bgcolor01,
        },
      },
    ],
  };
  if (!rastermap) {
    style.layers = style.layers.filter((l) => l.id != 'rastermap');
  }

  return style;
}
