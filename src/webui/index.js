import MapWidget from '../../../liveplaya-js'

// console.log("In kiosk index 2!");

// if (window) {
//     window.liveplaya = {Map: MapWidget};
// }

    var lowZoom = 14.6;
    var highZoom = 16;

    var map = new liveplaya.Map(document.getElementById('map'), {
        wantsFocus: true,
        zoomLevel: lowZoom,
        rasterTiles: 'https://cartodb-basemaps-{s}.global.ssl.fastly.net/dark_all/{z}/{x}/{y}.png',
        backgroundColor: '#000',
        outlineColor: '#bc560f',
        highlightColor: '#00ff00',
        mutedColor: '#00cc00',
        featureColor: function() {return '#00cc00';},
        showPoiStatus: false,
    })
    .addGeojsonDataUrl( document.location.origin + '/api/v0/features/', 1)
    .addGeojsonData({"type": "FeatureCollection", "features": [
            {
              "type": "Feature",
              "geometry": null,
              "id": "tgecko",
              "properties": {
                    "vehicle": "artcar",
                    "name": "TechnoGecko",
                    "description": "",
                    "tracker": "aprs/tgecko"
              }
            },
            {
              "type": "Feature",
              "geometry": null,
              "id": "guppy",
              "properties": {
                    "vehicle": "artcar",
                    "name": "Guppy",
                    "description": "",
                    "tracker": "aprs/dfgupy"
              }
            },
            {
              "type": "Feature",
              "geometry": null,
              "id": "keggy",
              "properties": {
                    "vehicle": "artcar",
                    "name": "Keggy",
                    "description": "",
                    "tracker": "aprs/dfkegy"
              }
            },
        ]});

    var list = new liveplaya.List(document.getElementById('list'), {
        data: map.data,
        filter: (f) => f.vehicleKind == 'artcar' || f.kind == 'person' || (f.kind == 'aprs' && showAllStations),
    });

    var track = null;
    var currViewIdx = 0;
    var timeoutId = null;
    var log = [];
    var showLog = true;
    var showAllStations = true;

    var update = function() {
        if (track) {
            var feat = map.data.features.find(function(f) {return f.id == track;});
            if (feat) {
                map.setView(feat.coords, highZoom);
            }
        } else {
            map.setView([-119.2066, 40.7866], lowZoom);
        }

        // var vehicleInfo = map.data.features
        //  .filter(function(f) {return (f.kind == 'vehicle' || (showAllStations && f.kind == 'aprs')) && f.lastseen;})
        //  .map(function(f) {return (f.id == track ? '* ' : '  ') + f.name  + ': ' + f.status(map.data.city, {verbose:true});})
        //  .join('\n');
        // document.getElementById('info').textContent = vehicleInfo;           

        var logContent = showLog ? map.data.features
            .filter(function(f) {return f.kind == 'aprs'})
            .sort(function(a,b) {return b.lastseen - a.lastseen})
            .slice(0, 10)
            .reverse()
            .map(function(f) {return f.lastseen.toLocaleTimeString() + ' ' + f.rawPacket})
            .join('\n') : '';   
        document.getElementById('log').textContent = logContent;            
    }

    var nextView = function() {
        var toTrack = [null].concat(map.data.features
                .filter(function(f) {return (f.kind == 'vehicle' || (showAllStations && f.kind == 'aprs')) && f.coords;})
                .map(function(f) {return f.id}));

        track = toTrack[currViewIdx++ % toTrack.length];
        update();
        clearTimeout(timeoutId);
        timeoutId = setTimeout(nextView, 20000);
    }

    var handleKeyEvent = function(e) {
        if (e.key == 'g') {
            map.showGrid = !map.showGrid;
        }       
        if (e.key == 'l') {
            showLog = !showLog;
            update();
        }       
        if (e.key == 'a') {
            showAllStations = !showAllStations;
            update();
        }       
        if (e.key == ' ') {
            nextView();
        }       
    };

    document.onkeydown = handleKeyEvent;
    map.data.on('update', update);

    nextView();