use crate::{
    bmorg,
    brc::{self, BlackRockCity},
    clockpos::ClockPos,
    err::Error,
    util::{time::*, units::*},
};

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

impl std::convert::TryInto<BurningMan> for bmorg::Data {
    type Error = crate::err::Error;

    fn try_into(self) -> std::result::Result<BurningMan, Self::Error> {
        let bmorg::Data {
            theme,
            gates_open_at,
            gis,
        } = self;
        let city = match gis {
            None => None,
            Some(bmorg::GisData {
                center,
                centerlines,
                outlines,
                ..
            }) => {
                let mut city = BlackRockCity::new(center);
                // First set up all ring streets so that we know spans for
                // radial streets
                for f in &centerlines.features {
                    match &f.properties {
                        bmorg::CenterlinesFeatureProps::Ring {
                            name,
                            radius_ft,
                            width_ft,
                        } => {
                            city = city.ring(name.clone(), ft2m(*radius_ft), ft2m(*width_ft))?;
                        }
                        _ => (),
                    }
                }
                let esp_radius_m = city.esplanade()?.radius_m();
                let mid_ring_radius_m = city
                    .ring_by_id(brc::RingId::FStreet)
                    .ok_or(Error::msg("no F-street"))?
                    .radius_m();
                let last_ring_radius_m = city.last_ring()?.radius_m();

                // Then do radial streets
                for f in &centerlines.features {
                    match &f.properties {
                        bmorg::CenterlinesFeatureProps::Radial { name, width_ft } => {
                            if let Ok(clock) = name.parse::<ClockPos>() {
                                let inner_radius_m = if clock.min() % 10 == 0 {
                                    esp_radius_m
                                } else {
                                    mid_ring_radius_m
                                };
                                city = city.radial(
                                    clock,
                                    inner_radius_m,
                                    last_ring_radius_m,
                                    ft2m(*width_ft),
                                )?;
                            }
                        }
                        _ => (),
                    }
                }
                for f in outlines.features {
                    let mut f = f.clone();
                    f.set_property("liveplaya", "bmorg-street-outlines");
                    city = city.other(f)?;
                }
                Some(city.finish()?)
            }
        };

        Ok(BurningMan {
            theme: Some(theme),
            gates_open_at,
            city,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let data = crate::bmorg::data_for_year(2023).unwrap();
        let bm: BurningMan = data.try_into().unwrap();
        assert_eq!(
            bm.city()
                .unwrap()
                .ring(crate::brc::RingId::DStreet)
                .unwrap()
                .name()
                .to_owned(),
            "Dingbat"
        );
    }
}
