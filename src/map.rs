use crate::aprs;
use crate::{
    aprs::Log,
    brc::BlackRockCity,
    err::{Error, Result},
    util::{geo::*, time::*},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};

#[derive(Debug)]
pub struct Data {
    city: BlackRockCity,
    aprslog: aprs::Log,
}

impl Data {
    pub fn new() -> Self {
        let centerlines =
            serde_json::from_str::<crate::bmorg::Document>(crate::bmorg::BRC2023_CENTERLINES)
                .unwrap();
        let city: BlackRockCity = centerlines.try_into().unwrap();
        let aprslog = aprs::Log::new();
        Data { aprslog, city }
    }

    // pub fn update_base(&mut self, map: Map) {
    //     self.basemap = map;
    // }

    pub fn update_aprs(&mut self, received: Timestamp, data: String) -> Result<()> {
        self.aprslog.push(received, data)
    }

    pub fn snapshot(&self, feat: Option<String>) -> Snapshot {
        Snapshot {
            feature: feat,
            data: self,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Snapshot<'a> {
    feature: Option<String>,
    data: &'a Data,
}
impl<'a> Snapshot<'a> {
    pub fn name(&self) -> String {
        "Black Rock City 2023".into()
    }

    pub fn timestamp(&self) -> Timestamp {
        Timestamp::now()
    }

    pub fn center(&self) -> Point {
        self.data.city.center()
    }

    // pub fn bounds(&self) -> BBox {
    //     self.data.city.bounds()
    // }

    pub fn description(&self) -> Option<String> {
        Some(format!(
            "Seen {} APRS stations",
            self.data.aprslog.station_count()
        ))
    }

    pub fn aprs_log(&self) -> &aprs::Log {
        &self.data.aprslog
    }

    fn to_geojson(&self) -> geojson::FeatureCollection {
        let mut features: Vec<geojson::Feature> = vec![];
        for (_ts, pr) in self.data.aprslog.last_positions() {
            let loc = pr.pos.location;
            features.push(geojson::Feature {
                geometry: Some(geojson::Geometry {
                    bbox: None,
                    value: loc.into(),
                    foreign_members: None,
                }),
                bbox: None,
                id: None,
                foreign_members: None,
                properties: Some(
                    serde_json::json!({
                        "liveplaya": "poi",
                        "poi": "beacon",
                        "name": pr.src_callsign,
                        "location": self.data.city.rgeocode(loc),
                    })
                    .as_object()
                    .unwrap()
                    .clone(),
                ),
            });
        }

        geojson::FeatureCollection {
            bbox: None,
            foreign_members: Some(
                json!({
                    "name": self.name(),
                    "description": self.description(),
                    "timeStr": self.timestamp().to_iso_string_utc(),
                    "bearingDeg": 45.,
                    "center": self.center(),
                    // "bounds": BBox;
                    // "zoom": number;

                    "log": self.data.aprslog.recent_entries().collect::<Vec<&aprs::LogEntry>>(),
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
            features,
        }
    }

    pub fn to_json(&self) -> JsonValue {
        serde_json::to_value(self.to_geojson()).unwrap()
    }

    pub fn to_maplibre_style(&self) -> Option<JsonValue> {
        Some(json!(self.to_geojson()))
    }
}
