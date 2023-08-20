use crate::err::{Error, Result};
use serde::{Deserialize, Serialize};

type DateTime = ::time::OffsetDateTime;
type Month = ::time::Month;
pub type Duration = std::time::Duration;

// pub const MIDNIGHT: Time = Time::from_hms(0, 0, 0).unwrap();

/// Millisecond timestamp
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp {
    unix_millis: u64,
}

impl Timestamp {
    pub const MIN: Timestamp = Timestamp::from_unix_millis(0);
    pub const MAX: Timestamp = Timestamp::from_unix_millis(u64::MAX);
    pub const BM_EPOCH: Timestamp = Timestamp::from_unix_millis(519807600000);

    pub fn now() -> Self {
        let t = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap();
        Timestamp {
            unix_millis: t.as_millis() as u64,
        }
    }

    pub const fn from_unix_millis(unix_millis: u64) -> Self {
        Timestamp { unix_millis }
    }

    pub fn from_iso_string(s: impl AsRef<str>) -> Result<Self> {
        let s = s.as_ref();
        let fmt = ::time::format_description::well_known::Rfc3339;
        let t = DateTime::parse(&s, &fmt)
            .map_err(|e| Error::Other(format!("can't parse ISO-8601 time: {}: {}", s, e)))?;
        Self::try_from(t)
    }

    pub fn from_calendar_pdt(
        year: u16,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Result<Timestamp> {
        Self::from_calendar_and_offset(year, month, day, hour, minute, second, -7)
    }

    pub fn from_calendar_utc(
        year: u16,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
    ) -> Result<Timestamp> {
        Self::from_calendar_and_offset(year, month, day, hour, minute, second, 0)
    }

    pub fn from_calendar_and_offset(
        year: u16,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        utc_offset_hrs: i8,
    ) -> Result<Timestamp> {
        let offset = ::time::UtcOffset::from_hms(utc_offset_hrs, 0, 0)
            .map_err(|e| Error::Other(format!("bad UTC offset {}: {}", utc_offset_hrs, e)))?;

        let month: Month = month
            .try_into()
            .map_err(|_| Error::TimeOutOfRange(format!("invalid month: {}", month)))?;

        let dt = DateTime::UNIX_EPOCH
            .replace_year(year as i32)
            .map_err(|_| Error::TimeOutOfRange(format!("invalid year: {}", year)))?
            .replace_month(month)
            .expect("month is valid as we already checked it")
            .replace_day(day)
            .map_err(|_| Error::TimeOutOfRange(format!("invalid day: {}", day)))?
            .replace_hour(hour)
            .map_err(|_| Error::TimeOutOfRange(format!("invalid hour: {}", hour)))?
            .replace_minute(minute)
            .map_err(|_| Error::TimeOutOfRange(format!("invalid minute: {}", minute)))?
            .replace_second(second)
            .map_err(|_| Error::TimeOutOfRange(format!("invalid second: {}", second)))?
            .to_offset(offset);

        Timestamp::try_from(dt)
    }

    pub fn parse(s: impl AsRef<str>) -> Result<Self> {
        Self::from_iso_string(s)
    }

    pub fn to_offset_hrs(&self, hrs: i8) -> Result<Self> {
        let offset = ::time::UtcOffset::from_hms(hrs, 0, 0)
            .map_err(|e| Error::Other(format!("bad UTC offset {}: {}", hrs, e)))?;
        let datetime = DateTime::from(*self).to_offset(offset);
        Self::try_from(datetime)
    }

    pub const fn as_unix_millis(&self) -> u64 {
        self.unix_millis
    }

    pub fn year(&self) -> u16 {
        DateTime::from(*self).year() as u16
    }

    pub fn intersects(&self, span: Timespan) -> bool {
        span.includes(*self)
    }

    pub fn to_iso_string_utc(&self) -> String {
        let fmt = ::time::format_description::well_known::Rfc3339;
        // let fmt =
        //     ::time::format_description::parse("[year]-[month]-[day]T[hour]:[minute]:[second]Z")
        //         .unwrap();
        DateTime::from(*self).format(&fmt).unwrap()
    }

    pub const fn duration_between(self, rhs: Self) -> Duration {
        Duration::from_millis(self.millis_between(rhs))
    }

    pub const fn millis_between(self, rhs: Self) -> u64 {
        let from = self.as_unix_millis();
        let to = rhs.as_unix_millis();
        let diff = from.abs_diff(to);
        diff
    }

    pub fn add(self, rhs: Duration) -> Result<Self> {
        let from = self.as_unix_millis();
        let diff = rhs.as_millis();
        if from as u128 + diff > u64::MAX as u128 {
            Err(Error::OutOfRange {
                msg: format!(
                    "adding {} ms to {} overflows timestamp",
                    rhs.as_millis(),
                    self
                ),
            })
        } else {
            Ok(Self::from_unix_millis(from + diff as u64))
        }
    }

    pub const fn saturating_add(self, rhs: Duration) -> Self {
        let from = self.as_unix_millis();
        let diff = rhs.as_millis();
        if diff > u64::MAX as u128 {
            Self::MAX
        } else {
            Self::from_unix_millis(from + diff as u64)
        }
    }

    pub fn sub(self, rhs: Duration) -> Result<Self> {
        let from = self.as_unix_millis();
        let diff = rhs.as_millis();
        if from as u128 > diff {
            Err(Error::OutOfRange {
                msg: format!(
                    "subtracting {} ms from {} underflows timestamp",
                    rhs.as_millis(),
                    self
                ),
            })
        } else {
            Ok(Self::from_unix_millis(from - diff as u64))
        }
    }

    pub const fn saturating_sub(self, rhs: Duration) -> Self {
        let from = self.as_unix_millis();
        let diff = rhs.as_millis();
        if from as u128 > diff {
            Self::MIN
        } else {
            Self::from_unix_millis(from - diff as u64)
        }
    }

    pub const fn min(self, rhs: Self) -> Self {
        if self.as_unix_millis() <= rhs.as_unix_millis() {
            self
        } else {
            rhs
        }
    }

    pub const fn max(self, rhs: Self) -> Self {
        if self.as_unix_millis() > rhs.as_unix_millis() {
            self
        } else {
            rhs
        }
    }
}

impl TryFrom<DateTime> for Timestamp {
    type Error = self::Error;

    fn try_from(dt: DateTime) -> Result<Self> {
        (dt.unix_timestamp_nanos() / 1_000_000)
            .try_into()
            .map_err(|_| Error::TimeOutOfRange(dt.to_string()))
            .map(|ts| Self::from_unix_millis(ts))
    }
}

impl TryFrom<String> for Timestamp {
    type Error = Error;

    fn try_from(v: String) -> Result<Self> {
        Self::from_iso_string(v)
    }
}

impl From<Timestamp> for DateTime {
    fn from(ts: Timestamp) -> Self {
        DateTime::from_unix_timestamp_nanos(ts.as_unix_millis() as i128 * 1_000_000)
            .expect("any timestamp expected to be representable as DateTime")
    }
}

impl std::fmt::Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_iso_string_utc())
    }
}

impl serde::Serialize for Timestamp {
    fn serialize<S>(&self, ser: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let v = self.to_iso_string_utc();
        ser.serialize_str(&v)
    }
}

impl<'de> serde::Deserialize<'de> for Timestamp {
    fn deserialize<D>(deser: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = <String>::deserialize(deser)?;
        Self::from_iso_string(v).map_err(|e| serde::de::Error::custom(e))
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Timespan {
    start: Timestamp,
    end: Timestamp,
}

impl Timespan {
    pub const MAX: Self = Timespan::from_two_timestamps(Timestamp::MIN, Timestamp::MAX);

    pub fn new(ts1: Timestamp, ts2: Timestamp) -> Self {
        Self::from_two_timestamps(ts1, ts2)
    }

    pub const fn from_two_timestamps(ts1: Timestamp, ts2: Timestamp) -> Self {
        let start = ts1.min(ts2);
        let end = ts1.max(ts2);
        Self { start, end }
    }

    pub const fn moment(ts: Timestamp) -> Self {
        Self { start: ts, end: ts }
    }

    pub fn from_timestamp_and_ms(ts: Timestamp, duration_ms: u32) -> Result<Self> {
        Ok(Self::new(
            ts,
            ts.add(Duration::from_millis(duration_ms as u64))?,
        ))
    }

    pub fn from_timestamp_and_days(ts: Timestamp, duration_days: u32) -> Result<Self> {
        Ok(Self::new(ts, ts.add(Duration::from_days(duration_days))?))
    }

    pub fn from_timestamp_and_duration(ts: Timestamp, duration: Duration) -> Result<Self> {
        Ok(Self::new(ts, ts.add(duration)?))
    }

    pub fn entire_year(y: u16) -> Result<Self> {
        let start = Timestamp::from_calendar_utc(y, 1, 1, 0, 0, 0)?;
        let end = Timestamp::from_calendar_utc(y, 12, 31, 23, 59, 59)?;
        Ok(Self::from_two_timestamps(start, end))
    }

    pub fn week_until_now() -> Self {
        let now = Timestamp::now();
        let dur = Duration::from_millis(7 * 24 * 3600 * 1000 as u64);
        Self::from_two_timestamps(Timestamp::now().saturating_sub(dur), now)
    }

    pub const fn start(&self) -> Timestamp {
        self.start
    }

    pub const fn end(&self) -> Timestamp {
        self.end
    }

    #[deprecated(since = "0.3.1", note = "please use `start` instead")]
    pub fn start_time(&self) -> Timestamp {
        self.start()
    }

    #[deprecated(since = "0.3.1", note = "please use `end` instead")]
    pub fn end_time(&self) -> Timestamp {
        self.end()
    }

    #[deprecated(since = "0.3.1", note = "please use `start` instead")]
    pub const fn start_ts(&self) -> Timestamp {
        self.start()
    }

    #[deprecated(since = "0.3.1", note = "please use `end` instead")]
    pub fn end_ts(&self) -> Timestamp {
        self.end()
    }

    pub const fn duration(&self) -> Duration {
        self.end.duration_between(self.start)
    }

    pub const fn duration_ms(&self) -> u64 {
        self.start().millis_between(self.end())
    }

    pub fn overlaps(&self, other: Timespan) -> bool {
        let max_start = self
            .start()
            .as_unix_millis()
            .max(other.start().as_unix_millis());
        let min_end = self
            .end()
            .as_unix_millis()
            .min(other.end().as_unix_millis());
        max_start <= min_end
    }

    pub fn includes(&self, ts: Timestamp) -> bool {
        ts >= self.start() && ts < self.end()
    }

    pub fn merge(&self, other: &Self) -> Self {
        let start = self.start().max(other.start());
        let end = self.end().min(other.end());
        Self::new(start, end)
    }
}

impl std::fmt::Display for Timespan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let dur = self.duration_ms();
        let pretty_dur = if dur as i128 > i64::MAX as i128 {
            String::new()
        } else {
            format!(" ({})", ::time::Duration::milliseconds(dur as i64))
        };
        write!(f, "{}..{}{}", &self.start(), self.end(), pretty_dur)
    }
}

pub trait DurationExt {
    fn from_days(n: u32) -> Self;
    fn whole_days(self) -> u64;
}

impl DurationExt for Duration {
    fn from_days(n: u32) -> Self {
        Self::from_secs((n as u64) * 24 * 3600)
    }

    fn whole_days(self) -> u64 {
        self.as_secs() / (24 * 3600)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert() {
        let dt = DateTime::from(Timestamp::BM_EPOCH);
        assert_eq!(dt.year(), 1986);
        assert_eq!(dt.month(), Month::June);
        assert_eq!(dt.day(), 22);

        let ts = Timestamp::try_from(dt).unwrap();
        assert_eq!(ts, Timestamp::BM_EPOCH);
    }

    #[test]
    fn test_add_timestamp() {
        let ts = Timestamp::from_calendar_utc(2022, 2, 2, 2, 2, 2).unwrap();
        let dur = Duration::from_secs(24 * 3600);
        assert_eq!(
            ts.add(dur).unwrap(),
            Timestamp::from_calendar_utc(2022, 2, 3, 2, 2, 2).unwrap()
        );
    }

    #[test]
    fn test_subtract_timestamp() {
        let ts1 = Timestamp::from_calendar_utc(2022, 2, 2, 2, 2, 2).unwrap();
        let ts2 = Timestamp::from_calendar_utc(2022, 2, 2, 2, 3, 2).unwrap();
        // let dur = Duration::days(1);
        assert_eq!(ts2.millis_between(ts1), 60000);
    }

    #[test]
    fn test_entire_year_timespan() {
        let span = Timespan::entire_year(2019).unwrap();
        assert_eq!(span.start().to_string(), "2019-01-01T00:00:00Z");
        assert_eq!(span.end().to_string(), "2019-12-31T23:59:59Z");
        assert_eq!(span.duration().whole_days(), 364);
    }

    #[test]
    fn test_timespan_overlap() {
        let ts1 = Timestamp::from_calendar_utc(2019, 12, 01, 0, 0, 0).unwrap();
        let ts2 = Timestamp::from_calendar_utc(2018, 12, 01, 0, 0, 0).unwrap();
        let span1 = Timespan::entire_year(2019).unwrap();
        let span2 = Timespan::entire_year(2022).unwrap();
        let span3 = Timespan::from_timestamp_and_days(ts1, 7).unwrap();
        let span4 = Timespan::from_timestamp_and_days(ts1, 60).unwrap();
        let span5 = Timespan::from_timestamp_and_days(ts2, 60).unwrap();

        assert_eq!(span1.overlaps(span2), false);
        assert_eq!(span2.overlaps(span1), false);

        assert_eq!(span1.overlaps(span3), true);
        assert_eq!(span3.overlaps(span1), true);
        assert_eq!(span1.overlaps(span4), true);
        assert_eq!(span4.overlaps(span1), true);
        assert_eq!(span1.overlaps(span5), true);
        assert_eq!(span5.overlaps(span1), true);
    }
}
