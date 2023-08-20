use crate::util::{geo::Point, time::Timestamp};

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "Type")]
pub enum CenterlinesFeatureProps {
    Ring {
        #[serde(rename = "Label_Text")]
        name: String,

        #[serde(rename = "Distance")]
        radius_ft: f64,

        #[serde(rename = "Width")]
        width_ft: f64,
    },

    Radial {
        #[serde(rename = "Name")]
        name: String,

        // #[serde(rename="Hour")]
        // hour: String,

        // #[serde(rename="Minute")]
        // minute: String,
        #[serde(rename = "Width")]
        width_ft: f64,
    },

    #[serde(other)]
    Other,
}

#[derive(Debug, serde::Deserialize)]
pub struct CenterlinesFeature {
    //pub geometry: geojson::Geometry,
    pub properties: CenterlinesFeatureProps,
}

#[derive(Debug, serde::Deserialize)]
pub struct Centerlines {
    pub features: Vec<CenterlinesFeature>,
}

pub struct GisData {
    pub center: Point,
    pub centerlines: Centerlines,
    pub outlines: geojson::FeatureCollection,
}

pub struct Data {
    pub theme: String,
    pub gates_open_at: Timestamp,
    pub gis: Option<GisData>,
}
