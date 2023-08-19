use crate::{
    err::{Error, Result},
    util::{geo::Point, time::Timestamp},
};

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

const BRC2023_CENTERLINES: &'static str = std::include_str!("../data/2023/Street_Centerlines.json");
const BRC2023_OUTLINES: &'static str = std::include_str!("../data/2023/Street_Outlines.json");

pub fn data_for_year(year: u16) -> Result<Data> {
    if year != 2023 {
        Err(Error::msg(format!("no BMOrg data for {}", year)))
    } else {
        Ok(Data {
            theme: "Animalia".to_string(),
            gates_open_at: Timestamp::from_calendar_pdt(year,8,27,0,0,0).unwrap(),
            gis: Some(GisData {
                center: Point::unchecked(-119.2035, 40.7864),
                centerlines: serde_json::from_str(BRC2023_CENTERLINES).unwrap(),
                outlines: serde_json::from_str(BRC2023_OUTLINES).unwrap(),
            }),
        })
    }
}
