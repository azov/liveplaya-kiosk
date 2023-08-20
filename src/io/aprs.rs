use crate::{
    err::{Error,Result},
    util::{geo::*, units::ft2m},
    motion::Position,
};
use serde::{Deserialize, Serialize};
use ::aprs::Packet as PacketTrait;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Packet {
    Position(PositionReport),
}
impl Packet {
    pub fn parse(data: impl AsRef<str>) -> Result<Self> {
        let data = data.as_ref();
        let res = ::fap::Packet::new(data.to_string()).map_err(|e| Error::AprsParse {
            what: data.to_string(),
            why: e.to_string(),
        })?;

        let srccall = res.source();
        let pos = res.position();
        let speed = res.speed();
        let course = res.course();
        let _symbol = res.symbol();
        let _dstcsll = res.destination();
        let _altitude = res.altitude();
        let comment = res.comment();

        let pos = match pos {
            None => {
                return Err(Error::AprsParse {
                    what: data.to_string(),
                    why: "not a position update".into(),
                })
            }
            Some(pos) => Ok::<Position, Error>(Position {
                location: Point::new(pos.longitude as f64, pos.latitude as f64)?,
                location_error_m: pos.precision.map(|v| ft2m(v.as_f64())),
                heading_deg: course.map(|v| v.as_f64()),
                speed_mps: speed.map(|v| ::aprs::MetersPerSecond::from(v).as_f64()),
            }),
        }?;

        Ok(Self::Position(PositionReport {
            src_callsign: srccall.into(),
            pos,
            // symbol,
            comment: comment.map(|v| v.into()),
        }))
    }

    pub fn srccall(&self) -> &str {
        match self {
            Packet::Position(PositionReport {
                src_callsign: srccall,
                ..
            }) => srccall.as_str(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionReport {
    pub src_callsign: String,
    pub pos: Position,
    // symbol: (u8,u8),
    pub comment: Option<String>,
}

