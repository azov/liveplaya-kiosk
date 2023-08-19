use crate::{
    clockpos::ClockPos,
    err::{Error, Result},
    util::geo::{normalize_angle, LineString, Point, Polygon},
};
use std::borrow::Cow;
use std::collections::HashMap;

#[derive(Debug)]
pub struct BlackRockCity {
    center: Point,
    bearing_deg: f64,
    ring_streets: HashMap<RingId, Ring>,
    radial_streets: Vec<Radial>,
    perimeter: Polygon,
    last_ring_street_id: RingId,
    other_feats: Vec<::geojson::Feature>,
}

impl BlackRockCity {
    pub fn new(center: Point) -> BrcBuilder {
        BrcBuilder::new(center)
    }

    pub fn center(&self) -> Point {
        self.center
    }

    pub fn bearing_deg(&self) -> f64 {
        self.bearing_deg
    }

    pub fn ring(&self, id: RingId) -> Option<&Ring> {
        self.ring_streets.get(&id)
    }

    pub fn rings(&self) -> impl Iterator<Item = &Ring> {
        self.ring_streets.values()
    }

    pub fn radials(&self) -> impl Iterator<Item = &Radial> {
        self.radial_streets.iter()
    }

    pub fn cstreets(&self) -> impl Iterator<Item = &Ring> {
        self.rings()
    }

    pub fn tstreets(&self) -> impl Iterator<Item = &Radial> {
        self.radials()
    }

    pub fn esplanade(&self) -> &Ring {
        self.esplanade_if_any().expect("no Esplanade")
    }

    fn esplanade_if_any(&self) -> Option<&Ring> {
        self.ring_streets.get(&RingId::Esplanade)
    }

    pub fn last_ring(&self) -> &Ring {
        self.last_ring_if_any().unwrap()
    }

    fn last_ring_if_any(&self) -> Option<&Ring> {
        self.ring_streets.get(&self.last_ring_street_id)
    }

    pub fn ring_by_name(&self, name: impl AsRef<str>) -> Result<&Ring> {
        let name = name.as_ref();
        let id = RingId::from_name(name)?;
        match self.ring_streets.get(&id) {
            Some(v) => Ok(v),
            None => Err(Error::Other(format!("ring street '{}' not found", name))),
        }
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

    pub fn other_features(&self) -> impl Iterator<Item=&::geojson::Feature> {
        self.other_feats.iter()
    }

    pub fn geocode(&self, cp: ClockPos, distance_m: f64) -> Point {
        self.center()
            .haversine_destination(cp.to_degrees() + self.bearing_deg(), distance_m)
    }

    pub fn rgeocode(&self, pt: Point) -> String {
        let center = self.center();
        let dist_m = pt.haversine_distance_m(center);
        if dist_m > 10000. {
            return "default world".into();
        }
        let deg = center.haversine_bearing_deg(pt);
        let clock = ClockPos::from_degrees(deg);
        format!("{} {:0} ft from man", clock, crate::util::units::ft2m(dist_m))
    }
}

#[derive(Debug)]
pub struct Ring {
    pub name: String,
    pub radius_m: f64,
    pub width_m: f64,
}

impl Ring {
    pub fn name(&self) -> Cow<str> {
        Cow::Borrowed(&self.name)
    }

    pub fn radius_m(&self) -> f64 {
        self.radius_m
    }

    pub fn width_m(&self) -> f64 {
        self.width_m
    }

    pub fn from_deg(&self, city: &BlackRockCity) -> f64 {
        normalize_angle(ClockPos::C02_00.to_degrees() + city.bearing_deg())
    }

    pub fn to_deg(&self, city: &BlackRockCity) -> f64 {
        normalize_angle(ClockPos::C10_00.to_degrees() + city.bearing_deg())
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

#[derive(Debug)]
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

pub struct BrcBuilder {
    city: BlackRockCity, // partially built city
}

impl BrcBuilder {
    pub fn new(center: impl Into<Point>) -> Self {
        let center = center.into();
        Self {
            city: BlackRockCity {
                center,
                bearing_deg: 45.,
                ring_streets: HashMap::new(),
                radial_streets: Vec::new(),
                perimeter: Polygon::cyclic(center, 1000.0, 5),
                last_ring_street_id: RingId::Esplanade,
                other_feats: Vec::new(),
            },
        }
    }

    pub fn bearing_deg(mut self, v: f64) -> Self {
        self.city.bearing_deg = v;
        self
    }

    pub fn north_south_axis(mut self, v: &ClockPos) -> Self {
        self.city.bearing_deg = v.to_degrees() - 90.0;
        self
    }

    pub fn perimeter(mut self, v: Polygon) -> Result<Self> {
        self.city.perimeter = v;
        Ok(self)
    }

    pub fn ring(mut self, name: String, radius_m: f64, width_m: f64) -> Result<Self> {
        let id = RingId::from_name(&name)?;
        if self.city.last_ring_street_id < id {
            self.city.last_ring_street_id = id;
        }
        self.city.ring_streets.insert(
            id,
            Ring {
                name,
                radius_m,
                width_m,
            },
        );
        Ok(self)
    }

    pub fn radial(
        mut self,
        direction: ClockPos,
        inner_radius_m: f64,
        outer_radius_m: f64,
        width_m: f64,
    ) -> Result<Self> {
        self.city.radial_streets.push(Radial {
            direction,
            inner_radius_m,
            outer_radius_m,
            width_m,
        });
        Ok(self)
    }

    pub fn other(mut self, f: ::geojson::Feature) -> Result<Self> {
        self.city.other_feats.push(f);
        Ok(self)
    }

    pub fn finish(self) -> Result<BlackRockCity> {
        Ok(self.city)
    }

    pub fn esplanade(&self) -> Result<&Ring> {
        self.city
            .esplanade_if_any()
            .ok_or(Error::msg("no Esplanade"))
    }

    pub fn ring_by_id(&self, id: RingId) -> Option<&Ring> {
        self.city.ring(id)
    }

    pub fn last_ring(&self) -> Result<&Ring> {
        self.city
            .last_ring_if_any()
            .ok_or(Error::msg("need at least one ring street"))
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub enum RingId {
    Esplanade,
    AStreet,
    BStreet,
    CStreet,
    DStreet,
    EStreet,
    FStreet,
    GStreet,
    HStreet,
    IStreet,
    JStreet,
    KStreet,
    LStreet,
    MStreet,
    NStreet,
}
impl RingId {
    fn from_name(name: impl AsRef<str>) -> Result<Self> {
        let name = name.as_ref();
        let lc = name.to_ascii_lowercase();
        if lc == "esplanade" || lc == "esp" {
            return Ok(RingId::Esplanade);
        }
        match lc.chars().next() {
            Some(c) if c == 'a' => Ok(RingId::AStreet),
            Some(c) if c == 'b' => Ok(RingId::BStreet),
            Some(c) if c == 'c' => Ok(RingId::CStreet),
            Some(c) if c == 'd' => Ok(RingId::DStreet),
            Some(c) if c == 'e' => Ok(RingId::EStreet),
            Some(c) if c == 'f' => Ok(RingId::FStreet),
            Some(c) if c == 'g' => Ok(RingId::GStreet),
            Some(c) if c == 'h' => Ok(RingId::HStreet),
            Some(c) if c == 'i' => Ok(RingId::IStreet),
            Some(c) if c == 'j' => Ok(RingId::JStreet),
            Some(c) if c == 'k' => Ok(RingId::KStreet),
            Some(c) if c == 'l' => Ok(RingId::LStreet),
            Some(c) if c == 'm' => Ok(RingId::MStreet),
            Some(c) if c == 'n' => Ok(RingId::NStreet),
            Some(_) => Err(Error::Other(format!("invalid ring street name '{}'", name))),
            None => Err(Error::msg("ring street name can't be empty")),
        }
    }
}
