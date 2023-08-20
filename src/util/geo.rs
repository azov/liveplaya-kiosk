use crate::err::{Error, Result};
use serde::{Serialize, Deserialize};

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

pub type Rect = BBox;

#[cfg_attr(feature = "wasm", wasm_bindgen)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    lng: f64,
    lat: f64,
}

impl Point {
    pub const ZERO: Self = Self::unchecked(0., 0.);
    pub const BRD_CENTER: Self = Self::unchecked(-119.202740, 40.787030);
    pub const MIN: Self = Self::unchecked(-180., -90.);
    pub const MAX: Self = Self::unchecked(180., 90.);

    pub fn new(lng: f64, lat: f64) -> Result<Self> {
        Self::check_lng(lng)?;
        Self::check_lat(lat)?;
        Ok(Self::unchecked(lng, lat))
    }

    pub const fn unchecked(lng: f64, lat: f64) -> Self {
        Self { lng, lat }
    }

    pub fn from_latlng(latlng: (f64, f64)) -> Result<Self> {
        Self::new(latlng.1, latlng.0)
    }

    pub fn from_lnglat(lnglat: (f64, f64)) -> Result<Self> {
        Self::new(lnglat.0, lnglat.1)
    }

    pub const fn lng(&self) -> f64 {
        self.lng
    }

    pub const fn lat(&self) -> f64 {
        self.lat
    }

    pub const fn longitude(&self) -> f64 {
        self.lng()
    }

    pub const fn latitude(&self) -> f64 {
        self.lat()
    }

    pub const fn lnglat(&self) -> (f64, f64) {
        (self.lng(), self.lat())
    }

    pub const fn latlng(&self) -> (f64, f64) {
        (self.lat(), self.lng())
    }

    pub fn round_to_about_meter(&self) -> Self {
        Self::unchecked(
            Self::_round_to_about_meter(self.lng()),
            Self::_round_to_about_meter(self.lat()),
        )
    }

    pub fn haversine_destination(&self, bearing_deg: f64, dist_m: f64) -> Self {
        let orig = ::geo::Point::new(self.lng(), self.lat());
        let dest = ::geo::HaversineDestination::haversine_destination(&orig, bearing_deg, dist_m);
        Self::unchecked(dest.x(), dest.y())
    }

    pub fn haversine_distance_m(&self, other: Point) -> f64 {
        let orig = ::geo::Point::new(self.lng(), self.lat());
        let dest = ::geo::Point::new(other.lng(), other.lat());
        ::geo::HaversineDistance::haversine_distance(&orig, &dest)
    }

    pub fn haversine_bearing_deg(&self, other: Point) -> f64 {
        let orig = ::geo::Point::new(self.lng(), self.lat());
        let dest = ::geo::Point::new(other.lng(), other.lat());
        ::geo::algorithm::HaversineBearing::haversine_bearing(&orig, dest)
    }

    pub fn translate(&self, dlng: f64, dlat: f64) -> Self {
        let mut lng = self.lng() + dlng % 360.;
        let mut lat = self.lat() + dlat % 360.;
        if lng > Point::MAX.lng() {
            lng = lng - 360.;
        }
        if lng < Point::MIN.lng() {
            lng = lng + 360.;
        }
        if lat > Point::MAX.lat() {
            lat = lat - 180.;
        }
        if lat < Point::MIN.lat() {
            lat = lat + 180.;
        }
        Point::unchecked(lng, lat)
    }

    fn check_lat(lat: f64) -> Result<f64> {
        if lat >= Self::MIN.lat() && lat <= Self::MAX.lat() {
            Ok(lat)
        } else {
            Err(Error::BadLatitude(lat))
        }
    }

    fn check_lng(lng: f64) -> Result<f64> {
        if lng >= Self::MIN.lng() && lng <= Self::MAX.lng() {
            Ok(lng)
        } else {
            Err(Error::BadLongitude(lng))
        }
    }

    fn _round_to_about_meter(v: f64) -> f64 {
        let k = 100000.;
        (v * k).round() / k
    }
}

#[cfg(never)]
#[cfg_attr(feature = "wasm", wasm_bindgen)]
impl LngLat {
    #[cfg_attr(feature = "wasm", wasm_bindgen(constructor))]
    pub fn new_js(lng: f64, lat: f64) -> Result<LngLat> {
        Self::check_lng(lng)?;
        Self::check_lat(lat)?;
        Ok(Self(lng, lat))
    }

    #[cfg_attr(feature = "wasm", wasm_bindgen(js_name = "lng"))]
    pub fn lng_non_const(&self) -> f64 {
        self.0
    }

    #[cfg_attr(feature = "wasm", wasm_bindgen(js_name = "lat"))]
    pub fn lat_non_const(&self) -> f64 {
        self.1
    }
}

impl std::fmt::Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.5},{:.5}", self.lng(), self.lat())
    }
}

impl TryFrom<(f64, f64)> for Point {
    type Error = Error;

    fn try_from(v: (f64, f64)) -> std::result::Result<Self, Self::Error> {
        Self::new(v.0, v.1)
    }
}

impl Into<(f64, f64)> for Point {
    fn into(self) -> (f64, f64) {
        self.lnglat()
    }
}

impl Into<::geojson::Value> for Point {
    fn into(self) -> ::geojson::Value {
        ::geojson::Value::Point(vec![self.lng(), self.lat()])
    }
}

impl serde::Serialize for Point {
    fn serialize<S>(&self, ser: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use ::serde::ser::SerializeTuple;
        let v = self.lnglat();
        let mut state = ser.serialize_tuple(2)?;
        state.serialize_element(&v.0)?;
        state.serialize_element(&v.1)?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for Point {
    fn deserialize<D>(deser: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = <(f64, f64)>::deserialize(deser)?;
        Self::from_lnglat(v).map_err(|e| serde::de::Error::custom(e))
    }
}




#[derive(Debug, Clone)]
pub struct BBox {
    min: Point,
    max: Point,
}

impl BBox {
    pub fn new(pt1: Point, pt2: Point) -> Self {
        Self::from_two_points(pt1, pt2)
    }

    pub fn from_two_points(pt1: Point, pt2: Point) -> Self {
        let min = Point::unchecked(
            f64::min(pt1.lng(), pt2.lng()),
            f64::min(pt1.lat(), pt1.lat()),
        );
        let max = Point::unchecked(
            f64::max(pt1.lng(), pt2.lng()),
            f64::max(pt1.lat(), pt2.lat()),
        );
        BBox { min, max }
    }

    pub fn from_center_and_radius(center: Point, radius_m: f64) -> Self {
        let north = center.haversine_destination(0.0, radius_m);
        let south = center.haversine_destination(180.0, radius_m);
        let east = center.haversine_destination(90.0, radius_m);
        let west = center.haversine_destination(270.0, radius_m);
        Self::from_args(south.lng(), west.lat(), north.lng(), east.lat()).unwrap()
    }

    pub fn from_args(lng1: f64, lat1: f64, lng2: f64, lat2: f64) -> Result<Self> {
        Self::from_tuple((lng1, lat1, lng2, lat2))
    }

    pub fn from_tuple(v: (f64, f64, f64, f64)) -> Result<Self> {
        let min = Point::try_from((f64::min(v.0, v.2), f64::min(v.1, v.3)))?;
        let max = Point::try_from((f64::max(v.0, v.2), f64::max(v.1, v.3)))?;
        Ok(Self { min, max })
    }

    pub fn from_str(v: impl AsRef<str>) -> Result<Self> {
        let v = v.as_ref();
        let mut parts = v.split(',');
        match (parts.next(), parts.next(), parts.next(), parts.next()) {
            (Some(x), Some(y), Some(z), Some(w)) => {
                if let (Ok(x_val), Ok(y_val), Ok(z_val), Ok(w_val)) =
                    (x.parse(), y.parse(), z.parse(), w.parse())
                {
                    Self::from_tuple((x_val, y_val, z_val, w_val))
                } else {
                    Err(Error::Other(format!(
                        "expected all 4 numbers to be floats, got '{}'",
                        v
                    )))
                }
            }
            _ => Err(Error::Other(format!(
                "expected comma-separated numbers, got '{}'",
                v
            ))),
        }
    }

    pub fn min(&self) -> Point {
        self.min
    }

    pub fn max(&self) -> Point {
        self.max
    }

    pub fn center(&self) -> Point {
        Point::unchecked(
            (self.min.lng() + self.max.lng()) / 2.,
            (self.min.lat() + self.max.lat()) / 2.,
        )
    }

    /// A 4-element tuple compatible with GeoJson representation
    pub fn to_tuple(&self) -> (f64, f64, f64, f64) {
        (
            self.min.lng(),
            self.min.lat(),
            self.max.lng(),
            self.max.lat(),
        )
    }

    /// A 4-element vector compatible with GeoJson representation
    pub fn to_vec(&self) -> Vec<f64> {
        vec![
            self.min.lng(),
            self.min.lat(),
            self.max.lng(),
            self.max.lat(),
        ]
    }

    /// Calculate max bounding box with given center that completely fits inside
    /// this bounding box
    #[cfg(never)]
    pub fn fit_around_center(&self, c: LngLat) -> Self {
        let c = self.center();
        let w1 = c.lng() - self.min().lng();
        let w2 = self.max().lng() - c.lng();
        let h1 = c.lat() - self.min().lat();
        let h2 = self.max().lat() - c.lat();
        let w = w1.min(w2);
        let h = h1.min(h2);

        let min = Point::new(c.lng() - w, c.lat() - h).unwrap();
        let max = Point::new(c.lng() - w, c.lat() - h).unwrap();
        Self::new(min, max)
    }

    pub fn contains(&self, pt: Point) -> bool {
        pt.lng() >= self.min().lng()
            && pt.lng() < self.max().lng()
            && pt.lat() >= self.min().lat()
            && pt.lat() < self.max().lat()
    }

    pub const MAX: BBox = BBox {
        min: Point::MIN,
        max: Point::MAX,
    };
}

impl std::fmt::Display for BBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = self.to_tuple();
        write!(f, "{:.5},{:.5},{:.5},{:.5}", v.0, v.1, v.2, v.3)
    }
}

impl serde::Serialize for BBox {
    fn serialize<S>(&self, ser: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use ::serde::ser::SerializeTuple;
        let v = self.to_tuple();
        let mut state = ser.serialize_tuple(4)?;
        state.serialize_element(&v.0)?;
        state.serialize_element(&v.1)?;
        state.serialize_element(&v.2)?;
        state.serialize_element(&v.3)?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for BBox {
    fn deserialize<D>(deser: D) -> std::result::Result<BBox, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = <(f64, f64, f64, f64)>::deserialize(deser)?;
        BBox::from_tuple(v).map_err(|e| serde::de::Error::custom(e))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct LineString(Vec<Point>);

impl LineString {
    pub fn new(points: Vec<Point>) -> Self {
        Self(points)
    }

    /// Creates an arc with given center, radius and starting/ending angles.
    pub fn arc(center: Point, from_deg: f64, to_deg: f64, radius_m: f64) -> Self {
        interpolate_arc(center, radius_m, from_deg, to_deg, 100)
    }

    pub fn from_latlngs(points: impl Iterator<Item = (f64, f64)>) -> Result<Self> {
        points
            .map(|v| Point::try_from((v.1, v.0)))
            .collect::<Result<Vec<Point>>>()
            .map(|v| Self(v))
    }

    pub fn from_latlng_refs<'a>(points: impl Iterator<Item = &'a (f64, f64)>) -> Result<Self> {
        points
            .map(|v| Point::try_from((v.1, v.0)))
            .collect::<Result<Vec<Point>>>()
            .map(|v| Self(v))
    }

    pub fn from_lnglat_refs<'a>(points: impl Iterator<Item = &'a (f64, f64)>) -> Result<Self> {
        points
            .map(|v| Point::try_from(v.clone()))
            .collect::<Result<Vec<Point>>>()
            .map(|v| Self(v))
    }

    pub fn translated(&self, dlng: f64, dlat: f64) -> Self {
        LineString::new(
            self.0
                .iter()
                .map(|v| v.clone().translate(dlng, dlat))
                .collect(),
        )
    }

    pub fn reversed(mut self) -> Self {
        self.0.reverse();
        self
    }

    pub fn extended_with(mut self, mut other: LineString) -> Self {
        self.0.append(&mut other.0);
        self
    }

    pub fn closed(mut self) -> Self {
        if let (Some(first_pt), Some(last_pt)) = (self.0.first(), self.0.last()) {
            if first_pt != last_pt {
                self.0.push(*first_pt);
            }
        }
        self
    }
}

impl Into<::geojson::Value> for LineString {
    fn into(self) -> ::geojson::Value {
        ::geojson::Value::LineString(
            self.0
                .into_iter()
                .map(|pt| vec![pt.lng(), pt.lat()])
                .collect(),
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct MultiLineString(Vec<LineString>);

impl MultiLineString {
    #[allow(dead_code)]
    pub fn new(segments: Vec<LineString>) -> Self {
        Self(segments)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Polygon {
    exterior: LineString,
    holes: Vec<LineString>,
}

impl Polygon {
    pub fn new(exterior: LineString, holes: Vec<LineString>) -> Self {
        let exterior = exterior.closed();
        let holes = holes.into_iter().map(|v| v.closed()).collect();
        Self { exterior, holes }
    }

    pub fn from_geojson_value(value: &geojson::Value) -> Result<Self> {
        match value {
            ::geojson::Value::Polygon(polygon) => {
                let exterior_vec = if let Some(vec) = polygon.first() {
                    vec
                } else {
                    return Err(Error::msg("polygon must have non-empty exterior"));
                };
                let mut exterior_points = Vec::with_capacity(exterior_vec.len());
                for pt in exterior_vec {
                    exterior_points.push(Point::new(pt[0], pt[1])?);
                }
                let exterior = LineString::new(exterior_points);
                let mut holes = Vec::new();
                for hole in &polygon[1..] {
                    let mut points = Vec::with_capacity(hole.len());
                    for pt in hole  {
                        points.push(Point::new(pt[0], pt[1])?);
                    }
                    holes.push(LineString::new(points))
                }
                return Ok(Self::new(exterior, holes));
            },
            _ => Err(Error::Other(format!("expected a polygon, got {}", value))),
        }
    }

    pub fn cyclic(center: Point, radius_m: f64, vertex_cnt: usize) -> Self {
        if vertex_cnt < 3 {
            panic!(
                "cyclic polygon must have at least 3 vertexes, got {}",
                vertex_cnt
            );
        }
        let exterior = interpolate_arc(center, radius_m, 0., 360., vertex_cnt);
        Self::new(exterior, Vec::new())
    }

    pub fn from_latlngs(points: impl Iterator<Item = (f64, f64)>) -> Result<Self> {
        LineString::from_latlngs(points).map(|ls| Self::new(ls, vec![]))
    }
}

impl Into<::geojson::Value> for Polygon {
    fn into(self) -> ::geojson::Value {
        let mut coords = Vec::new();
        coords.push(
            self.exterior
                .0
                .iter()
                .map(|pt| vec![pt.lng(), pt.lat()])
                .collect(),
        );
        for v in self.holes {
            coords.push(v.0.into_iter().map(|pt| vec![pt.lng(), pt.lat()]).collect());
        }
        ::geojson::Value::Polygon(coords)
    }
}

impl TryFrom<::geojson::Geometry> for Polygon {
    type Error = crate::err::Error;
    fn try_from(v: ::geojson::Geometry) -> std::result::Result<Self, Self::Error> {
        v.value.try_into()
    }
}

impl TryFrom<::geojson::Value> for Polygon {
    type Error = crate::err::Error;

    fn try_from(value: ::geojson::Value) -> std::result::Result<Self, Self::Error> {
        Polygon::from_geojson_value(&value)
    }
}



/// Generate points along the circle arc
fn interpolate_arc(
    center: Point,
    radius_m: f64,
    from_deg: f64,
    to_deg: f64,
    npoints: usize,
) -> LineString {
    let (from_deg, delta) = normalize_arc_range(from_deg, to_deg, npoints);
    let mut points = Vec::with_capacity(npoints);
    for i in 0..npoints {
        points.push(center.haversine_destination(from_deg + i as f64 * delta, radius_m));
    }
    LineString::new(points)
}

/// Convert an angle to 0..360 range.
pub fn normalize_angle(deg: f64) -> f64 {
    let mut normalized = deg % 360.;
    if normalized < 0. {
        normalized += 360.
    }
    normalized
}

// Returns start angle and delta for arc interpolation
fn normalize_arc_range(from_deg: f64, to_deg: f64, npoints: usize) -> (f64, f64) {
    let direction = if from_deg <= to_deg { 1. } else { -1. };
    let from_deg = normalize_angle(from_deg);
    let mut to_deg = normalize_angle(to_deg);
    if to_deg <= from_deg {
        to_deg += 360.;
    }
    let delta = direction * (to_deg - from_deg) / (npoints - 1) as f64;
    (from_deg, delta)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_arc_range() {
        assert_eq!(normalize_arc_range(0., 0., 5), (0., 90.));
        assert_eq!(normalize_arc_range(0., 360., 5), (0., 90.));
        assert_eq!(normalize_arc_range(360., 0., 5), (0., -90.));
        assert_eq!(normalize_arc_range(90., 180., 6), (90., 18.));
        assert_eq!(normalize_arc_range(180., 90., 6), (180., -54.));
        assert_eq!(normalize_arc_range(45., -45., 6), (45., -54.));
        assert_eq!(normalize_arc_range(45., 360. - 45., 6), (45., 54.));
        assert_eq!(normalize_arc_range(-45., 45., 6), (315., 18.));
    }

    #[test]
    fn test_invalid_lnglat() {
        assert!(matches!(
            Point::try_from((-190., 0.)),
            Err(Error::BadLongitude(v)) if v == -190.
        ));

        assert!(matches!(
            Point::try_from((0., 100.)),
            Err(Error::BadLatitude(v)) if v == 100.
        ));

        assert!(matches!(
            Point::try_from((0., f64::INFINITY)),
            Err(Error::BadLatitude(v)) if v == f64::INFINITY
        ));

        assert!(matches!(
            Point::try_from((f64::NEG_INFINITY, 0.)),
            Err(Error::BadLongitude(v)) if v == f64::NEG_INFINITY
        ));
    }

    #[test]
    fn test_polygon_closes_itself() {
        let ext = LineString::from_latlngs(
            vec![
                (40.783341, -119.233011),
                (40.807777, -119.216715),
                (40.803538, -119.181098),
                (40.776488, -119.175400),
                (40.764008, -119.207478),
            ]
            .into_iter(),
        )
        .unwrap();
        let p = Polygon::new(ext, vec![]);
        assert_eq!(p.exterior.0.len(), 6);
    }

    #[test]
    fn test_interpolate_arc() {
        let ls = interpolate_arc(Point::BRD_CENTER, 5000., 180., 90., 5);
        // let gjs: ::geojson::Value = ls.into();
        // println!("{}", gjs);
        assert_eq!(ls.0.len(), 5);
    }

    #[test]
    #[cfg(never)]
    fn test_bbox_fit() {
        let bbox = BBox::from_args(-20., -20., 20., 20.).unwrap();
        let c = Point::new(10., 10.).unwrap();
        assert_eq!(bbox.fit_around_center(c).center(), c);
    }
}

#[cfg(never)]
mod prj {
    fn project_point(pt: LngLat) -> ::geo::Coord {
        ::geo::Coord {
            x: pt.lng(),
            y: pt.lat(),
        }
    }

    fn unproject_point(pt: ::geo::Coord) -> LngLat {
        Point::try_from((pt.x, pt.y)).unwrap()
    }

    fn project_line(line: LineString) -> ::geo::LineString {
        ::geo::LineString::new(line.0.into_iter().map(project_point).collect())
    }

    fn unproject_line(line: ::geo::LineString) -> LineString {
        LineString::new(line.into_iter().map(unproject_point).collect())
    }

    fn project_polygon(poly: Polygon) -> ::geo::Polygon {
        let Polygon { exterior, holes } = poly;
        ::geo::Polygon::new(
            project_line(exterior),
            holes.into_iter().map(project_line).collect(),
        )
    }

    fn unproject_polygon(poly: ::geo::Polygon) -> Polygon {
        let (exterior, holes) = poly.into_inner();
        Polygon {
            exterior: unproject_line(exterior),
            holes: holes.into_iter().map(unproject_line).collect(),
        }
    }
}

pub type LngLat = (f64, f64);
