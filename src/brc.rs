use crate::{
    bmorg,
    clockpos::ClockPos,
    err::{Error, Result},
    util::{
        geo::{normalize_angle, LineString, Point, Polygon},
        time::Timestamp,
        units::ft2m,
    },
};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlackRockCity {
    center: Point,
    gates_open: Timestamp,
    bearing_deg: f64,
    rings: Vec<Ring>,
    radials: Vec<Radial>,
    perimeter: Polygon,
    other_features: Vec<::geojson::Feature>,
}

impl BlackRockCity {
    pub fn from_bmorg_data(
        gates_open: Timestamp,
        golden_stake: Point,
        bearing_deg: f64,
        extent: geojson::FeatureCollection,
        centerlines: bmorg::Centerlines,
        outlines: Option<geojson::FeatureCollection>,
    ) -> Result<Self> {
        let center = golden_stake;

        // First process all ring streets
        let mut rings = Vec::new();
        let mut ring_names_seen = std::collections::HashSet::new();
        for f in &centerlines.features {
            match &f.properties {
                bmorg::CenterlinesFeatureProps::Ring {
                    name,
                    radius_ft,
                    width_ft,
                } => {
                    if !ring_names_seen.contains(name) {
                        ring_names_seen.insert(name);
                        rings.push(Ring {
                            name: name.clone(),
                            radius_m: ft2m(*radius_ft),
                            width_m: ft2m(*width_ft),
                        });
                    }
                }
                _ => (),
            }
        }
        rings.sort_by_key(|r| (r.radius_m * 1000.) as u64);
        if rings[0].name.as_str() != "Esplanade" {
            return Err(Error::msg("first ring must be an Esplanade"));
        }
        if rings.len() < 7 {
            return Err(Error::msg("must have at least 7 rings (Esp to F)"));
        }
        for i in 1..rings.len() {
            let name = &rings[i].name;
            let first_char = name.chars().next();
            if let Some(first_char) = first_char {
                let expected_char = (b'A' + i as u8 - 1) as char;
                if first_char != expected_char {
                    return Err(Error::Other(format!(
                        "ring #{} name must start with {}, got {}",
                        i, expected_char, name
                    )));
                }
            } else {
                return Err(Error::Other(format!(
                    "ring names must not be empty; bad ring #{}",
                    i
                )));
            }
        }
        let esp_radius_m = rings[0].radius_m;
        let fstreet_radius_m = rings[6].radius_m;
        let last_ring = &rings[rings.len() - 1];

        // Then do radial streets
        let mut radials = Vec::new();
        for f in &centerlines.features {
            match &f.properties {
                bmorg::CenterlinesFeatureProps::Radial { name, width_ft } => {
                    if let Ok(direction) = name.parse::<ClockPos>() {
                        let inner_radius_m = if direction.min() % 10 == 0 {
                            esp_radius_m
                        } else {
                            fstreet_radius_m
                        };
                        radials.push(Radial {
                            direction,
                            inner_radius_m,
                            outer_radius_m: last_ring.radius_m,
                            width_m: ft2m(*width_ft),
                        });
                    } else {
                        // do nothing. BMorg includes some odd radials, so we'll just skip them
                    }
                }
                _ => (),
            }
        }

        // Then do perimeter
        let perimeter = match extent.features.first().and_then(|f| f.geometry.as_ref()) {
            Some(g) => Ok(Polygon::from_geojson_value(&g.value)?),
            _ => Err(Error::msg(
                "extent must contain a polygon with perimeter points as first feature",
            )),
        }?;

        // Finally include any other features
        let mut other_features = Vec::new();
        if let Some(fc) = outlines {
            for mut f in fc.features.into_iter() {
                if f.properties.as_ref().map(|v| !v.contains_key("liveplaya")).unwrap_or(true) {
                    f.set_property("liveplaya", "bmorg-street-outlines");
                }
                other_features.push(f);
            }
        }
        Ok(BlackRockCity {
            center,
            gates_open,
            bearing_deg,
            rings,
            radials,
            perimeter,
            other_features,
        })
    }

    pub fn year(&self) -> u16 {
        self.gates_open.year()
    }

    pub fn center(&self) -> Point {
        self.center
    }

    pub fn bearing_deg(&self) -> f64 {
        self.bearing_deg
    }

    pub fn rings(&self) -> impl Iterator<Item = &Ring> {
        self.rings.iter()
    }

    pub fn radials(&self) -> impl Iterator<Item = &Radial> {
        self.radials.iter()
    }

    pub fn cstreets(&self) -> impl Iterator<Item = &Ring> {
        self.rings()
    }

    pub fn tstreets(&self) -> impl Iterator<Item = &Radial> {
        self.radials()
    }

    pub fn esplanade(&self) -> &Ring {
        &self.rings[0]
    }

    pub fn a_street(&self) -> &Ring {
        &self.rings[1]
    }

    pub fn last_ring(&self) -> &Ring {
        &self.rings[self.rings.len() - 1]
    }

    pub fn ring_by_index(&self, idx: usize) -> Option<&Ring> {
        self.rings.get(idx)
    }

    pub fn ring_by_name(&self, name: impl AsRef<str>) -> Result<&Ring> {
        let name = name.as_ref().to_ascii_lowercase();
        if name == "esp" || name == "esplanade" {
            return Ok(self.esplanade());
        }
        let first_char = if let Some(c) = name.chars().next() {
            c
        } else {
            return Err(Error::Other(format!("can't get ring by empty name")));
        };
        let idx = (first_char as u8 - b'a') + 1;
        self.ring_by_index(idx as usize)
            .ok_or(Error::Other(format!("no {} ring", name)))
    }

    pub fn camping_area(&self) -> Cow<Polygon> {
        let inner = self.esplanade().center_line(self).into_owned();
        let outer = self.last_ring().center_line(self).into_owned();
        let exterior = inner.extended_with(outer.reversed());
        Cow::Owned(Polygon::new(exterior, vec![]))
    }

    pub fn perimeter(&self) -> Cow<Polygon> {
        Cow::Borrowed(&self.perimeter)
    }

    pub fn other_features(&self) -> impl Iterator<Item = &::geojson::Feature> {
        self.other_features.iter()
    }

    pub fn geocode(&self, cp: ClockPos, distance_m: f64) -> Point {
        self.center()
            .haversine_destination(cp.to_degrees() + self.bearing_deg(), distance_m)
    }
 
    pub const DEFAULT_WORLD_THRESHOLD_M : f64 = 16000.;

    pub fn rgeocode(&self, pt: Point) -> String {
        let center = self.center();
        let dist_m = pt.haversine_distance_m(center);
        if dist_m > Self::DEFAULT_WORLD_THRESHOLD_M {
            return "default world".into();
        }
        let deg = center.haversine_bearing_deg(pt) - self.bearing_deg;
        let clock = ClockPos::from_degrees(deg);
        let esp_rad_m = self.esplanade().radius_m();
        let aring_rad_m = self.a_street().radius_m();
        let mut prev_ring_rad_m = esp_rad_m - (aring_rad_m - esp_rad_m) / 2.;

        if dist_m > prev_ring_rad_m && clock > ClockPos::TWO && clock < ClockPos::TEN {
            for ring in self.rings() {
                let this_ring_rad_m = ring.radius_m();
                if dist_m < (this_ring_rad_m + prev_ring_rad_m) / 2. {
                    return format!("{} & {}", clock, ring.abbr());
                }
                prev_ring_rad_m = this_ring_rad_m;
            }
            return "near Black Rock City".into();
        }
        format!("{} & {:.0}'", clock, crate::util::units::m2ft(dist_m))
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ring {
    pub name: String,
    pub radius_m: f64,
    pub width_m: f64,
}

impl Ring {
    pub fn name(&self) -> Cow<str> {
        Cow::Borrowed(&self.name)
    }

    pub fn abbr(&self) -> &str {
        if self.name.to_ascii_lowercase().as_str() == "esplanade" {
            return &self.name[..3];
        } else {
            return &self.name[..1];
        }
    }

    pub fn radius_m(&self) -> f64 {
        self.radius_m
    }

    pub fn width_m(&self) -> f64 {
        self.width_m
    }

    pub fn from_deg(&self, city: &BlackRockCity) -> f64 {
        normalize_angle(ClockPos::TWO.to_degrees() + city.bearing_deg())
    }

    pub fn to_deg(&self, city: &BlackRockCity) -> f64 {
        normalize_angle(ClockPos::TEN.to_degrees() + city.bearing_deg())
    }

    pub fn center_line(&self, city: &BlackRockCity) -> Cow<LineString> {
        Cow::Owned(LineString::arc(
            city.center(),
            self.from_deg(city),
            self.to_deg(city) + 360.,
            self.radius_m(),
        ))
    }

    fn outer_edge(&self, city: &BlackRockCity) -> Cow<LineString> {
        Cow::Owned(LineString::arc(
            city.center(),
            self.from_deg(city),
            self.to_deg(city) + 360.,
            self.radius_m() + self.width_m() / 2.,
        ))
    }

    fn inner_edge(&self, city: &BlackRockCity) -> Cow<LineString> {
        Cow::Owned(LineString::arc(
            city.center(),
            self.from_deg(city),
            self.to_deg(city) + 360.,
            self.radius_m() - self.width_m() / 2.,
        ))
    }

    pub fn start_point(&self, city: &BlackRockCity) -> Point {
        city.center()
            .haversine_destination(self.from_deg(city), self.radius_m())
    }

    pub fn end_point(&self, city: &BlackRockCity) -> Point {
        city.center()
            .haversine_destination(self.to_deg(city), self.radius_m())
    }

    pub fn area(&self, city: &BlackRockCity) -> Cow<Polygon> {
        let inner = self.inner_edge(city).into_owned();
        let outer = self.outer_edge(city).into_owned();
        let exterior = inner.extended_with(outer.reversed());
        Cow::Owned(Polygon::new(exterior, vec![]))
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]

pub struct Radial {
    pub direction: ClockPos,
    pub width_m: f64,
    pub inner_radius_m: f64,
    pub outer_radius_m: f64,
}

impl Radial {
    pub fn name(&self) -> Cow<str> {
        self.direction.to_string().into()
    }

    pub fn direction(&self) -> ClockPos {
        self.direction
    }

    pub fn width_m(&self) -> f64 {
        self.width_m
    }

    pub fn inner_radius_m(&self) -> f64 {
        self.inner_radius_m
    }

    pub fn outer_radius_m(&self) -> f64 {
        self.outer_radius_m
    }

    pub fn center_line(&self, city: &BlackRockCity) -> Cow<LineString> {
        Cow::Owned(LineString::new(vec![
            self.start_point(city).into(),
            self.end_point(city).into(),
        ]))
    }

    pub fn start_point(&self, city: &BlackRockCity) -> Point {
        city.geocode(self.direction(), self.inner_radius_m())
    }

    pub fn end_point(&self, city: &BlackRockCity) -> Point {
        city.geocode(self.direction(), self.outer_radius_m())
    }

    pub fn area(&self, city: &BlackRockCity) -> Cow<Polygon> {
        let dir_deg = self.direction().to_degrees() + city.bearing_deg() + 90.;
        let w = self.width_m();
        let c = city.center();
        let d = c.haversine_destination(dir_deg, w / 2.);
        let dlng = d.lng() - c.lng();
        let dlat = d.lat() - c.lat();

        let center = self.center_line(city).into_owned();
        let left = center.translated(dlng, dlat);
        let right = center.translated(-dlng, -dlat);
        let exterior = left.extended_with(right.reversed());
        Cow::Owned(Polygon::new(exterior, vec![]))
    }
}
