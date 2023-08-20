use super::*;
use crate::{
    aprs,
    brc::BlackRockCity,
    io,
    util::time::Timestamp,
};
use serde_json::json;

impl Server {
    pub async fn view(&self /* , _feature: Option<String>, _bbox: Option<BBox>, _zoom: Option<f64> */) -> Result<io::user::View> {
        let city = &self.brc;
        let log = &self.aprs_cache;
        let now = Timestamp::now();
        let mut features: Vec<geojson::Feature> = vec![];

        for street in city.cstreets() {
            features.push(geojson::Feature {
                geometry: Some(geojson::Geometry {
                    bbox: None,
                    foreign_members: None,
                    value: street.center_line(city).into_owned().into(),
                }),
                bbox: None,
                id: None,
                foreign_members: None,
                properties: Some(as_map(serde_json::json!({
                    "liveplaya": "streetcenter",
                    "name": street.name(),
                }))),
            });
            features.push(geojson::Feature {
                geometry: Some(geojson::Geometry {
                    bbox: None,
                    foreign_members: None,
                    value: street.area(city).into_owned().into(),
                }),
                bbox: None,
                id: None,
                foreign_members: None,
                properties: Some(as_map(serde_json::json!({
                    "liveplaya": "street",
                }))),
            });
            features.push(geojson::Feature {
                geometry: Some(geojson::Geometry {
                    bbox: None,
                    foreign_members: None,
                    value: street.start_point(city).into(),
                }),
                bbox: None,
                id: None,
                foreign_members: None,
                properties: Some(as_map(serde_json::json!({
                    "cstreet": "start",
                    "name": street.name(),
                    "tandg": street.from_deg(city),
                }))),
            });
            features.push(geojson::Feature {
                geometry: Some(geojson::Geometry {
                    bbox: None,
                    foreign_members: None,
                    value: street.end_point(city).into(),
                }),
                bbox: None,
                id: None,
                foreign_members: None,
                properties: Some(as_map(serde_json::json!({
                    "cstreet": "end",
                    "name": street.name(),
                    "tandg": street.to_deg(city),
                }))),
            });
        }
        for radial in city.radials() {
            features.push(geojson::Feature {
                geometry: Some(geojson::Geometry {
                    bbox: None,
                    foreign_members: None,
                    value: radial.center_line(city).into_owned().into(),
                }),
                bbox: None,
                id: None,
                foreign_members: None,
                properties: Some(as_map(json!({
                    "liveplaya": "streetcenter",
                    "name": radial.name(),
                }))),
            });
            features.push(geojson::Feature {
                geometry: Some(geojson::Geometry {
                    bbox: None,
                    foreign_members: None,
                    value: radial.area(city).into_owned().into(),
                }),
                bbox: None,
                id: None,
                foreign_members: None,
                properties: Some(as_map(json!({
                    "liveplaya": "street",
                }))),
            });
            features.push(geojson::Feature {
                geometry: Some(geojson::Geometry {
                    bbox: None,
                    foreign_members: None,
                    value: radial.end_point(city).into(),
                }),
                bbox: None,
                id: None,
                foreign_members: None,
                properties: Some(as_map(json!({
                    "liveplaya": "radialend",
                    "name": radial.name(),
                    "dir": radial.direction().to_degrees(),
                }))),
            });
        }

        for (ts, pr) in log.last_positions() {
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
                        "headingDeg": pr.pos.heading_deg,
                        "location": city.rgeocode(loc),
                        "lastseen": *ts,
                    })
                    .as_object()
                    .unwrap()
                    .clone(),
                ),
            });
        }
        for f in city.other_features() {
            features.push(f.clone());
        }

        let data = geojson::FeatureCollection {
            bbox: None,
            foreign_members: None,
            features,
        };

        let mut logmsgs = Vec::new();
        for (id, ts, raw, parsed) in log.recent_entries() {
            match parsed {
                Ok(aprs::Packet::Position(_pr)) => {
                    logmsgs.push(io::user::LogMessage::Info {
                        id: *id,
                        time: *ts,
                        text: format!("{}", raw),
                    });
                }
                Err(e) => {
                    logmsgs.push(io::user::LogMessage::Error {
                        id: *id,
                        time: *ts,
                        text: format!("{}: {}", raw, e),
                    });
                }
            }
        }

        let mut last_positions = log
            .last_positions()
            .collect::<Vec<&(Timestamp, aprs::PositionReport)>>();

        last_positions.sort_by_key(|(ts, pr)| {
            let man_dist = pr.pos.location.haversine_distance_m(city.center());
            let near_brc = man_dist < BlackRockCity::DEFAULT_WORLD_THRESHOLD_M;
            let seen_recently = ts.duration_between(now).as_secs() < 3600 * 3;
            (!near_brc, !seen_recently, &pr.src_callsign)
        });

        let mut refs = Vec::new();
        for (ts, pr) in last_positions {
            let location = city.rgeocode(pr.pos.location);

            refs.push(io::user::FeatureRef::Beacon {
                name: pr.src_callsign.clone(),
                slug: format!("aprs/{}", pr.src_callsign.to_ascii_lowercase()),
                lastseen: *ts,
                location,
            })
        }

        let view = io::user::View {
            name: format!("Black Rock City {}", city.year()),
            description: Some(format!("Watching {} APRS stations.", log.station_count())),
            time: Timestamp::now(),
            map: Some(io::user::Map {
                bearing_deg: 45.,
                center: city.center().lnglat(),
                zoom: 12.8,
                data,
            }),
            log: logmsgs,
            refs,
        };

        Ok(view)
    }
}

fn as_map(v: serde_json::Value) -> serde_json::Map<String, serde_json::Value> {
    match v {
        serde_json::Value::Object(m) => m,
        _ => panic!("expected object value"),
    }
}
