use crate::{brc::BlackRockCity, util::time::*};

#[derive(Debug)]
pub struct BurningMan {
    gates_open_at: Timestamp,
    theme: Option<String>,
    city: Option<BlackRockCity>,
}

impl BurningMan {
    pub fn new(year: u16, theme: Option<String>) -> Self {
        let gates_open_at = Timestamp::from_calendar_pdt(year, 08, 27, 00, 00, 00).unwrap();
        Self {
            theme,
            gates_open_at,
            city: None,
        }
    }

    pub fn with_city(mut self, city: BlackRockCity) -> Self {
        self.city = Some(city);
        self
    }

    pub fn with_gates_open_at(mut self, time: Timestamp) -> Self {
        self.gates_open_at = time;
        self
    }

    pub fn year(&self) -> u16 {
        self.gates_open_at.year() as u16
    }

    pub fn theme(&self) -> Option<&str> {
        self.theme.as_ref().map(|v| v.as_str())
    }

    pub fn city(&self) -> Option<&BlackRockCity> {
        self.city.as_ref()
    }

    pub fn gates_open_at(&self) -> Timestamp {
        self.gates_open_at
    }
}
