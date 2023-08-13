use crate::core::{*, Error};
use serde::{Serialize, Deserialize};

const MINUTES_ON_CLOCK_FACE: i32 = 12 * 60;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ClockPos {
    hour: u8,
    minute: u8,
}

impl ClockPos {
    pub const C02_00: ClockPos = ClockPos { hour: 2, minute: 0 };
    pub const C06_00: ClockPos = ClockPos { hour: 6, minute: 0 };
    pub const C10_00: ClockPos = ClockPos { hour: 10, minute: 0 };
    
    pub fn hr(&self) -> u8 {
        self.hour
    }

    pub fn min(&self) -> u8 {
        self.minute
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

    pub fn to_degrees(&self) -> f64 {
        let mut mins = (self.hour as i32 * 60 + self.minute as i32) % MINUTES_ON_CLOCK_FACE;
        if mins > MINUTES_ON_CLOCK_FACE / 2 {
            mins = -(MINUTES_ON_CLOCK_FACE - mins);
        }
        normalize_angle(360.0 * (mins as f64) / MINUTES_ON_CLOCK_FACE as f64)
    }
}

impl Display for ClockPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{:02}", self.hour, self.minute)
    }
}

