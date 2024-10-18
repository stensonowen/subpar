use chrono::{DateTime, Utc};
use std::{fmt, ops, time};
use anyhow::{Result, anyhow, bail};
use crate::msg::Date;

// const FMT: &str = "%F_%T"; // 2020-12-31~14:30:00

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub struct Timestamp(DateTime<Utc>);

impl Timestamp {
    pub fn now() -> Self {
        Timestamp(Utc::now())
    }
    pub fn from_yyyymmdd(s: &str) -> Result<Self> {
        let (y, m, d) = match (s.get(0..4), s.get(4..6), s.get(6..)) {
            (Some(y), Some(m), Some(d)) => (y.parse()?, m.parse()?, d.parse()?),
            _ => bail!("Malformed yyyymmdd `{}`", s),
        };
        let date = chrono::NaiveDate::from_ymd_opt(y, m, d)
            .ok_or_else(|| anyhow!("bad date {y}/{m}/{d} in {s}"))?;
        let datetime = date.and_hms_opt(0, 0, 0)
            .ok_or_else(|| anyhow!("bad datetime {date} @ 00:00:00 in {s}"))?;
        Ok(Timestamp(DateTime::from_naive_utc_and_offset(datetime, Utc)))
    }
    // pub fn from_unix_utc(unix: u64) -> Self {
    //     let naive = chrono::NaiveDateTime::from_timestamp_opt(unix as i64, 0).unwrap();
    //     Timestamp(DateTime::from_utc(naive, Utc))
    // }
    pub fn as_unix_utc(&self) -> u64 {
        self.0.timestamp() as u64
    }
    pub fn ms_since_epoch(&self) -> i64 {
        self.0.timestamp_millis()
    }
    pub fn as_utc(&self) -> DateTime<Utc> {
        self.0
    }
    pub fn from_utc(dt: DateTime<Utc>) -> Self {
        Timestamp(dt)
    }
    pub fn plus(&self, delta: chrono::Duration) -> Self {
        Timestamp(self.0 + delta)
    }
    // pub fn parse(s: &str) -> Option<Self> {
    //     use chrono::TimeZone as _;
    //     Utc.datetime_from_str(s, FMT).ok().map(Timestamp)
    // }
    pub fn time(&self) -> String {
        self.0.format("%T").to_string()
    }
    pub fn into_chrono(&self) -> DateTime<Utc> {
        self.0
    }
    pub fn from_naive(dt: chrono::NaiveDateTime) -> Self {
        Timestamp(DateTime::from_naive_utc_and_offset(dt, chrono::Utc))
    }
    pub fn seconds_since(&self, earlier: &Self) -> i64 {
        (self.0 - earlier.0).num_seconds()
    }
    pub fn epoch() -> Self {
        Self(DateTime::from_timestamp(0, 0).unwrap())
    }
    pub fn from_unix(unix: i64) -> Self {
        Self(DateTime::from_timestamp(unix, 0).unwrap())
    }
    pub fn date(&self) -> Date {
        Date::new(self.0.date_naive())
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use chrono::{Datelike, Timelike};
        // uses LOCAL time, NOT utc
        let local = self.0.with_timezone(&chrono::offset::Local);
        let today = Timestamp::now().0.date_naive();
        let time = match local.second() {
            0 => local.format("%H:%M"),
            _ => local.format("%T"),
        };
        if local.date_naive() == today {
            write!(f, "{}", time)
        } else if local.year() == today.year() {
            write!(f, "{} {}", local.format("%m/%d"), time)
        } else {
            write!(f, "{} {}", local.format("%F"), time)
        }
    }
}

impl ops::Add<time::Duration> for Timestamp {
    type Output = Self;
    fn add(self, delta: time::Duration) -> Self {
        // Panics in case of 64-bit overflow
        let delta = chrono::Duration::from_std(delta).unwrap();
        Timestamp(self.0 + delta)
    }
}


impl ops::Sub for Timestamp {
    type Output = time::Duration;
    fn sub(self, rhs: Self) -> time::Duration {
        (self.0 - rhs.0).to_std().unwrap()
    }
}

impl ops::Sub<time::Duration> for Timestamp {
    type Output = Timestamp;
    fn sub(self, dur: time::Duration) -> Self {
        Timestamp(self.0 - dur)
    }
}
