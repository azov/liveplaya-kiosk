use crate::util::{geo::*, time::Timestamp, twoway};
use geojson;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Query {
    pub feature: Option<String>,
    pub bounds: Option<BBox>,
    pub zoom: Option<f64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub name: String,
    pub description: Option<String>,
    pub time: Timestamp,

    #[serde(flatten)]
    pub map: Option<Map>,

    pub refs: Vec<FeatureRef>,
    pub log: Vec<LogMessage>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Map {
    pub bearing_deg: f64,
    pub center: LngLat,
    pub zoom: f64,

    #[serde(flatten)]
    pub data: geojson::FeatureCollection,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum FeatureRef {
    Beacon{name: String, slug: String, location: String, lastseen: Timestamp},
}


#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(tag="level")]
pub enum LogMessage {
    Info{id: u64, time: Timestamp, text: String},
    Error{id: u64, time: Timestamp, text: String},
}

pub type QuerySender = twoway::Sender<Query, View>;
pub type QueryReceiver = twoway::Receiver<Query, View>;
