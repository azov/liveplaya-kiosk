use crate::core::*;

type DateTime = ::time::OffsetDateTime;
type Month = ::time::Month;
pub type Duration = ::time::Duration;

// pub const MIDNIGHT: Time = Time::from_hms(0, 0, 0).unwrap();

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Timestamp(u64);
/// Millisecond timestamp

impl Timestamp {
    pub const BM_EPOCH: Timestamp = Timestamp(519807600000);

    pub fn now() -> Self {
        let t = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap();
        Timestamp(t.as_millis() as u64)
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

    pub fn from_unix_millis(millis: u64) -> Self {
        Timestamp(millis)
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


    pub fn to_offset_hrs(&self, hrs: i8) -> Result<Self> {
        let offset = ::time::UtcOffset::from_hms(hrs, 0, 0)
            .map_err(|e| Error::Other(format!("bad UTC offset {}: {}", hrs, e)))?;
        let datetime = DateTime::from(*self).to_offset(offset);
        Self::try_from(datetime)
    }

    pub fn as_unix_millis(&self) -> u64 {
        self.0
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

}

impl TryFrom<DateTime> for Timestamp {
    type Error = self::Error;

    fn try_from(dt: DateTime) -> Result<Self> {
        (dt.unix_timestamp_nanos() / 1_000_000)
            .try_into()
            .map_err(|_| Error::TimeOutOfRange(dt.to_string()))
            .map(|ts| Self(ts))
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
        DateTime::from_unix_timestamp_nanos(ts.0 as i128 * 1_000_000)
            .expect("any timestamp expected to be representable as DateTime")
    }
}

impl std::ops::Add<Duration> for Timestamp {
    type Output = Timestamp;
    fn add(self, rhs: Duration) -> Self::Output {
        Self(self.0 + rhs.whole_milliseconds() as u64)
    }
}

impl std::ops::Sub<Duration> for Timestamp {
    type Output = Timestamp;
    fn sub(self, rhs: Duration) -> Self::Output {
        Self(self.0 - rhs.whole_milliseconds() as u64)
    }
}

impl std::ops::Sub<Timestamp> for Timestamp {
    type Output = Duration;
    fn sub(self, rhs: Timestamp) -> Self::Output {
        Duration::milliseconds(
            (self.0 - rhs.0)
                .try_into()
                .expect("timestamp difference is not out of range"),
        )
    }
}

impl std::fmt::Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_iso_string_utc())
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Timespan {
    ts: Timestamp,
    duration_ms: i64,
}

impl Timespan {
    pub const MAX: Self = Timespan::from_timestamp_and_ms(Timestamp(0), i64::MAX as u64);

    pub fn new(ts1: Timestamp, ts2: Timestamp) -> Self {
        Self::from_two_timestamps(ts1, ts2)
    }

    pub fn from_two_timestamps(ts1: Timestamp, ts2: Timestamp) -> Self {
        let start = ts1.min(ts2);
        let end = ts1.max(ts2);
        Self {
            ts: start,
            duration_ms: (end - start).whole_milliseconds() as i64,
        }
    }

    pub const fn moment(ts: Timestamp) -> Self {
        Self { ts, duration_ms: 0 }
    }

    pub const fn from_timestamp_and_ms(ts: Timestamp, duration_ms: u64) -> Self {
        Self {
            ts,
            duration_ms: duration_ms as i64,
        }
    }

    pub fn from_timestamp_and_days(ts: Timestamp, duration_days: i64) -> Self {
        Self {
            ts,
            duration_ms: (Duration::days(duration_days).as_seconds_f64() * 1000.0) as i64,
        }
    }

    pub fn try_from_timestamp_and_duration(
        ts: Timestamp,
        duration: Duration,
    ) -> Result<Self> {
        let duration_ms: i64 = duration.whole_milliseconds().try_into().map_err(|_| {
            Error::TimeOutOfRange(format!("timestamp: {}, duration: {}", ts, duration))
        })?;

        Ok(Self { ts, duration_ms })
    }

    pub fn entire_year(y: u16) -> Self {
        let start = Timestamp::from_calendar_utc(y, 1, 1, 0, 0, 0).unwrap();
        let end = Timestamp::from_calendar_utc(y, 12, 31, 23, 59, 59).unwrap();
        Self {
            ts: start,
            duration_ms: (1000. * (end - start).as_seconds_f64()) as i64,
        }
    }

    pub fn week_until_now() -> Self {
        Self {
            ts: Timestamp::now(),
            duration_ms: 7 * 24 * 3600 * 1000 as i64,
        }
    }

    pub fn start_time(&self) -> Timestamp {
        //DateTime::from(self.ts)
        self.start_ts()
    }

    pub fn end_time(&self) -> Timestamp {
        // DateTime::from(self.ts + Duration::milliseconds(self.duration_ms))
        self.end_ts()
    }

    pub const fn start_ts(&self) -> Timestamp {
        self.ts
    }

    pub fn end_ts(&self) -> Timestamp {
        self.ts + Duration::milliseconds(self.duration_ms)
    }

    pub const fn duration(&self) -> Duration {
        Duration::milliseconds(self.duration_ms)
    }

    pub fn startduration(&self) -> (Timestamp, Duration) {
        (self.start_time(), self.duration())
    }

    pub fn overlaps(&self, other: Timespan) -> bool {
        let max_start = self.start_ts().0.max(other.start_ts().0);
        let min_end = self.end_ts().0.min(other.end_ts().0);
        max_start <= min_end
    }

    pub const fn includes(&self, ts: Timestamp) -> bool {
        self.ts.0 <= ts.0 && self.ts.0 + self.duration_ms as u64 > ts.0
    }

    pub fn merge(&self, other: &Self) -> Self {
        let ts = Timestamp(std::cmp::min(self.start_ts().0, other.start_ts().0));
        let dur = std::cmp::max(self.end_ts().0, other.end_ts().0) - ts.0;
        Self {
            ts,
            duration_ms: dur as i64,
        }
    }
}

impl std::fmt::Display for Timespan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}..{} ({})",
            &self.start_time(),
            self.end_time(),
            self.duration()
        )
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
        let dur = Duration::days(1);
        assert_eq!(
            ts + dur,
            Timestamp::from_calendar_utc(2022, 2, 3, 2, 2, 2).unwrap()
        );
    }

    #[test]
    fn test_subtract_timestamp() {
        let ts1 = Timestamp::from_calendar_utc(2022, 2, 2, 2, 2, 2).unwrap();
        let ts2 = Timestamp::from_calendar_utc(2022, 2, 2, 2, 3, 2).unwrap();
        // let dur = Duration::days(1);
        assert_eq!(ts2 - ts1, Duration::seconds(60));
    }

    #[test]
    fn test_entire_year_timespan() {
        let span = Timespan::entire_year(2019);
        assert_eq!(span.start_time().to_string(), "2019-01-01T00:00:00Z");
        assert_eq!(span.end_time().to_string(), "2019-12-31T23:59:59Z");
        assert_eq!(span.duration().whole_days(), 364);
    }

    #[test]
    fn test_timespan_overlap() {
        let ts1 = Timestamp::from_calendar_utc(2019, 12, 01, 0, 0, 0).unwrap();
        let ts2 = Timestamp::from_calendar_utc(2018, 12, 01, 0, 0, 0).unwrap();
        let span1 = Timespan::entire_year(2019);
        let span2 = Timespan::entire_year(2022);
        let span3 = Timespan::from_timestamp_and_days(ts1, 7);
        let span4 = Timespan::from_timestamp_and_days(ts1, 60);
        let span5 = Timespan::from_timestamp_and_days(ts2, 60);

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
