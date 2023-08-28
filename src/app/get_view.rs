use super::*;
use crate::{
    aprs,
    brc::BlackRockCity,
    io,
    util::{geo::Point, time::Timestamp},
};
use serde_json::json;

#[derive(Debug, Clone)]
struct Poi {
    name: String,
    location: Point,
    location_str: String,
    heading_deg: Option<f64>,
    lastseen: Timestamp,
    near_brc: bool,
    seen_recently: bool,
    known: bool,
    favorite: bool,
    slug: String,
}

impl Server {
    pub async fn view(
        &self, /* , _feature: Option<String>, _bbox: Option<BBox>, _zoom: Option<f64> */
    ) -> Result<io::user::View> {
        let show_default_world = false;
        let city = &self.brc;
        let log = &self.aprs_cache;
        let now = Timestamp::now();
        let mut features: Vec<geojson::Feature> = vec![];

        let mut pois = log
            .last_positions()
            .map(|(ts, pr)| {
                let man_dist = pr.pos.location.haversine_distance_m(city.center());
                let heading_deg = pr.pos.heading_deg;
                let near_brc = man_dist < BlackRockCity::DEFAULT_WORLD_THRESHOLD_M;
                let seen_recently = ts.duration_between(now).as_secs() < 3600 * 3;
                let mut slug = format!("aprs/{}", pr.src_callsign.to_ascii_lowercase());
                let mut name = pr.src_callsign.to_string();
                let location = pr.pos.location;
                let location_str = city.rgeocode(pr.pos.location);
                let lastseen = *ts;
                let mut known = false;
                let mut favorite = false;

                if let Some(poi) = self.pois_by_call.get(pr.src_callsign.as_str()) {
                    known = true;
                    favorite = poi.is_favorite;
                    name = poi.name.to_string();
                    slug = poi.slug.to_string();
                }

                Poi {
                    name,
                    slug,
                    near_brc,
                    seen_recently,
                    location,
                    location_str,
                    lastseen,
                    known,
                    favorite,
                    heading_deg,
                }
            })
            .filter(|poi| show_default_world || poi.near_brc)
            .collect::<Vec<Poi>>();

        pois.sort_by_key(|poi| {
            (
                !poi.favorite,
                !poi.known,
                !poi.near_brc,
                !poi.seen_recently,
                //poi.name.as_str(),
            )
        });

        let mut priority = 0;
        for poi in pois.iter() {
            priority += 1;
            features.push(geojson::Feature {
                geometry: Some(geojson::Geometry {
                    bbox: None,
                    value: poi.location.into(),
                    foreign_members: None,
                }),
                bbox: None,
                id: None,
                foreign_members: None,
                properties: Some(
                    serde_json::json!({
                        "liveplaya": "poi",
                        "poi": "beacon",
                        "name": poi.name,
                        "headingDeg": poi.heading_deg,
                        "location": poi.location_str,
                        "lastseen": poi.lastseen,
                        "priority": priority,
                        "_fav": poi.favorite,
                        "_kno": poi.known,
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

        let mut refs = Vec::new();
        for poi in pois.iter() {
            refs.push(io::user::FeatureRef::Beacon {
                name: poi.name.to_string(),
                slug: poi.slug.to_string(),
                location: poi.location_str.to_string(),
                lastseen: poi.lastseen,
            });
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
