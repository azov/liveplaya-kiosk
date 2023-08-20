use crate::{
    err::{Error, Result},
    util::geo::normalize_angle,
};
use serde::{Deserialize, Serialize};

const MINUTES_ON_CLOCK_FACE: i32 = 12 * 60;
const DEGREES_IN_HR: f64 = 360.0 / 12.0;
const DEGREES_IN_MIN: f64 = 360.0 / (MINUTES_ON_CLOCK_FACE as f64);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ClockPos {
    hour: u8,
    minute: u8,
}

impl ClockPos {
    pub const NOON: ClockPos = ClockPos {
        hour: 12,
        minute: 0,
    };
    pub const TWO: ClockPos = ClockPos { hour: 2, minute: 0 };
    pub const THREE: ClockPos = ClockPos { hour: 3, minute: 0 };
    pub const SIX: ClockPos = ClockPos { hour: 6, minute: 0 };
    pub const NINE: ClockPos = ClockPos { hour: 9, minute: 0 };
    pub const TEN: ClockPos = ClockPos {
        hour: 10,
        minute: 0,
    };

    pub fn hr(&self) -> u8 {
        self.hour
    }

    pub fn min(&self) -> u8 {
        self.minute
    }

    pub fn new(hour: u16, minute: u16) -> Result<Self> {
        Self::from_hr_min(hour, minute)
    }

    pub fn from_hr_min(hour: u16, minute: u16) -> Result<Self> {
        if hour <= 12 && minute < 60 {
            let hour = if hour == 0 { 12 } else { hour };
            return Ok(ClockPos {
                hour: hour as u8,
                minute: minute as u8,
            });
        }
        Err(Error::BadClock(hour, minute))
    }

    pub fn unpack(packed: u16) -> Result<Self> {
        Self::from_hr_min(packed / 100, packed % 100)
    }

    pub fn from_degrees(deg: f64) -> Self {
        let deg = normalize_angle(deg);
        let hour = (deg / DEGREES_IN_HR) as u8;
        let minute = ((deg - (hour as f64) * DEGREES_IN_HR) / DEGREES_IN_MIN) as u8;
        Self { hour, minute }
    }

    pub fn to_degrees(&self) -> f64 {
        let mut mins = (self.hour as i32 * 60 + self.minute as i32) % MINUTES_ON_CLOCK_FACE;
        if mins > MINUTES_ON_CLOCK_FACE / 2 {
            mins = -(MINUTES_ON_CLOCK_FACE - mins);
        }
        normalize_angle(360.0 * (mins as f64) / MINUTES_ON_CLOCK_FACE as f64)
    }
}

impl std::str::FromStr for ClockPos {
    type Err = crate::err::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let hm = s
            .trim()
            .split(":")
            .map(|v| {
                v.parse::<u16>().map_err(|_| {
                    Error::Other(format!("expected value in the form of HH:MM, got {}", s))
                })
            })
            .collect::<Result<Vec<u16>>>()?;
        Self::from_hr_min(hm[0], hm[1])
    }
}

impl std::fmt::Display for ClockPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{:02}", self.hour, self.minute)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let c = "02:00".parse::<ClockPos>().unwrap();
        assert_eq!(c.hour, 2);
        assert_eq!(c.minute, 0);

        let c = "02:15".parse::<ClockPos>().unwrap();
        assert_eq!(c.hour, 2);
        assert_eq!(c.minute, 15);

        let c = " 2:15".parse::<ClockPos>().unwrap();
        assert_eq!(c.hour, 2);
        assert_eq!(c.minute, 15);

        let c = " 2:17 ".parse::<ClockPos>().unwrap();
        assert_eq!(c.hour, 2);
        assert_eq!(c.minute, 17);

        let c = "4:15".parse::<ClockPos>().unwrap();
        assert_eq!(c.hour, 4);
        assert_eq!(c.minute, 15);

        let c = "12:15".parse::<ClockPos>().unwrap();
        assert_eq!(c.hour, 12);
        assert_eq!(c.minute, 15);

        assert!(matches!("11:60".parse::<ClockPos>(), Err(_)));
        assert!(matches!("13:15".parse::<ClockPos>(), Err(_)));
        assert!(matches!("7::15".parse::<ClockPos>(), Err(_)));
        assert!(matches!("foo".parse::<ClockPos>(), Err(_)));
    }

    #[test]
    fn test_to_degrees() {
        assert_eq!(ClockPos::NOON.to_degrees(), 0.);
        assert_eq!(ClockPos::THREE.to_degrees(), 90.);
        assert_eq!(ClockPos::SIX.to_degrees(), 180.);
        assert_eq!(ClockPos::NINE.to_degrees(), 270.);
        assert_eq!(ClockPos::new(2, 15).unwrap().to_degrees(), 67.5);
    }

    #[test]
    fn test_from_degrees() {
        assert_eq!(ClockPos::from_degrees(0.), ClockPos::NOON);
        assert_eq!(ClockPos::from_degrees(90.), ClockPos::THREE);
        assert_eq!(ClockPos::from_degrees(180.), ClockPos::SIX);
        assert_eq!(ClockPos::from_degrees(270.), ClockPos::NINE);
        assert_eq!(ClockPos::from_degrees(67.5), ClockPos::new(2, 15).unwrap());
    }

    #[test]
    fn test_ordering() {
        assert!(ClockPos::NOON > ClockPos::THREE);
        assert!(ClockPos::TWO < ClockPos::THREE);
        assert!(ClockPos::new(2,15).unwrap() < ClockPos::new(2,16).unwrap());
        assert!(ClockPos::new(3,1).unwrap() > ClockPos::new(2,59).unwrap());
        assert!(ClockPos::new(11,59).unwrap() < ClockPos::new(12,0).unwrap());
        assert!(ClockPos::new(12,01).unwrap() > ClockPos::new(12,0).unwrap());
    }

    #[test]
    fn test_roundtrip_to_string() {
        let c1 = "2:15";
        let deg = c1.parse::<ClockPos>().unwrap().to_degrees();
        let c2 = ClockPos::from_degrees(deg).to_string();
        assert_eq!(c1, c2);
    }
}
