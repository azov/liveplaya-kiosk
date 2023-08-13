use crate::core::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub location: Point,
    pub location_error_m: Option<f64>,
    pub heading_deg: Option<f64>,
    pub speed_mps: Option<f64>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Probe {
    pub ts: Timestamp,
    pub pos: Position,
}