use crate::core::*;

#[derive(Debug)]
pub struct Data {
    aprslog: aprs::Log,
}

impl Data {
    pub fn new() -> Self {
        let aprslog = aprs::Log::new();
        Data { aprslog }
    }

    // pub fn update_base(&mut self, map: Map) {
    //     self.basemap = map;
    // }

    pub fn update_aprs(&mut self, received: Timestamp, data: String) -> Result<()> {
        self.aprslog.add(received, data)
    }

    pub fn snapshot(&self, feat: Option<String>) -> Snapshot {
        Snapshot { feature: feat, data: self }
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

    pub fn description(&self) -> Option<String> {
        Some(format!(
            "Seen {} APRS stations",
            self.data.aprslog.station_count()
        ))
    }

    fn to_geojson(&self) -> geojson::FeatureCollection {
        let mut features: Vec<geojson::Feature> = vec![];
        for (_ts, pr) in self.data.aprslog.positions() {
            features.push(geojson::Feature {
                geometry: Some(geojson::Geometry {
                    bbox: None,
                    value: pr.pos.location.into(),
                    foreign_members: None,
                }),
                bbox: None,
                id: None,
                foreign_members: None,
                properties: Some(serde_json::json!({
                    "liveplaya": "poi",
                    "poi": "beacon",
                    "name": pr.srccall,
                }).as_object().unwrap().clone()),
            });
        }

        geojson::FeatureCollection {
            bbox: None,
            foreign_members: None,
            features,
        }
    }

    pub fn to_maplibre_style(&self) -> Option<JsonValue> {
        Some(json!(self.to_geojson()))
    }

}
