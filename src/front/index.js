import * as maplibre from 'maplibre-gl';
import * as mapstyle from './mapstyle';

// console.log("In kiosk index 2!");

// if (window) {
//     window.liveplaya = {Map: MapWidget};
// }

    var lowZoom = 14.6;
    var highZoom = 16;
    var style = mapstyle.build(); 
    console.log("style:", style);
    const map = new maplibre.Map({
        container: "map",
        // style: 'https://demotiles.maplibre.org/style.json',
        style,
        center: [-119.203500, 40.786400],
        zoom: 13.5,
        bearing: 45,
      });

    var track = null;
    var currViewIdx = 0;
    var timeoutId = null;
    var log = [];
    var showLog = true;
    var showAllStations = true;

    // var update = function() {
    //     if (track) {
    //         var feat = map.data.features.find(function(f) {return f.id == track;});
    //         if (feat) {
    //             map.setView(feat.coords, highZoom);
    //         }
    //     } else {
    //         map.setView([-119.2066, 40.7866], lowZoom);
    //     }

    //     // var vehicleInfo = map.data.features
    //     //  .filter(function(f) {return (f.kind == 'vehicle' || (showAllStations && f.kind == 'aprs')) && f.lastseen;})
    //     //  .map(function(f) {return (f.id == track ? '* ' : '  ') + f.name  + ': ' + f.status(map.data.city, {verbose:true});})
    //     //  .join('\n');
    //     // document.getElementById('info').textContent = vehicleInfo;           

    //     var logContent = showLog ? map.data.features
    //         .filter(function(f) {return f.kind == 'aprs'})
    //         .sort(function(a,b) {return b.lastseen - a.lastseen})
    //         .slice(0, 10)
    //         .reverse()
    //         .map(function(f) {return f.lastseen.toLocaleTimeString() + ' ' + f.rawPacket})
    //         .join('\n') : '';   
    //     document.getElementById('log').textContent = logContent;            
    // }

    // var nextView = function() {
    //     var toTrack = [null].concat(map.data.features
    //             .filter(function(f) {return (f.kind == 'vehicle' || (showAllStations && f.kind == 'aprs')) && f.coords;})
    //             .map(function(f) {return f.id}));

    //     track = toTrack[currViewIdx++ % toTrack.length];
    //     update();
    //     clearTimeout(timeoutId);
    //     timeoutId = setTimeout(nextView, 20000);
    // }

    // var handleKeyEvent = function(e) {
    //     if (e.key == 'g') {
    //         map.showGrid = !map.showGrid;
    //     }       
    //     if (e.key == 'l') {
    //         showLog = !showLog;
    //         update();
    //     }       
    //     if (e.key == 'a') {
    //         showAllStations = !showAllStations;
    //         update();
    //     }       
    //     if (e.key == ' ') {
    //         nextView();
    //     }       
    // };

    // document.onkeydown = handleKeyEvent;
    // map.data.on('update', update);

    // nextView();


async function update() {
    try {
        const resp = await fetch("/api/v0/");
        const data = await resp.json();
        map.getSource("rtfeatures").setData(data.view.map);
        // console.log("got", data);
    }
    catch(e) {
        console.error(e);
    }
}

update();
setInterval(update, 1000);
