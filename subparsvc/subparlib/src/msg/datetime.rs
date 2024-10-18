use crate::Timestamp;
use anyhow::Context as _;
use std::{convert::TryInto as _, fmt, ops, str};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Date(chrono::NaiveDate);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Time {
    pub h: u8,
    pub m: u8,
    pub s: u8,
    pub offset: i8, // day
}

impl Date {
    pub fn new(date: chrono::NaiveDate) -> Self {
        Date(date)
    }
    pub fn make(y: i32, m: u32, d: u32) -> Self {
        Date(chrono::NaiveDate::from_ymd_opt(y, m, d).unwrap())
    }
    pub fn to_naive(self) -> chrono::NaiveDate {
        self.0
    }
}

impl Time {
    pub fn new(h: u8, m: u8, s: u8) -> Self {
        Time { h, m, s, offset: 0 }
    }
    pub fn new_with_offset(h: u8, m: u8, s: u8, offset: i8) -> Self {
        Time { h, m, s, offset }
    }
    pub fn from_trip_origin(s: &str) -> anyhow::Result<Self> {
        // 'hundredths of a minute past midnight'
        // can be negative or > 24*60*100
        let n: i32 = s.parse()?;
        const DAY: i32 = 24 * 60 * 100;
        let (n_norm, offset) = if n < 0 {
            (n + DAY, -1)
        } else if n >= DAY {
            (n - DAY, 1)
        } else {
            (n, 0)
        };
        let nf = n_norm as f64;
        let mins_total = nf / 100.0;
        let hrs = mins_total / 60.0;
        let mins = mins_total % 60.0;
        let secs = mins_total.fract() * 60.0;
        let h = (hrs as u64).try_into().context("hours")?;
        let m = (mins as u64).try_into().context("mins")?;
        let s = (secs as u64).try_into().context("secs")?;
        Ok(Self::new_with_offset(h, m, s, offset))
    }
    fn secs_since_last_mid(self) -> u64 {
        assert!(self.offset >= -1);
        let days = (1 + self.offset) as u64;
        let hours = days * 24 + self.h as u64;
        let mins = hours * 60 + self.m as u64;
        let secs = mins * 60 + self.s as u64;
        secs
    }
}

impl Date {}

impl ops::Sub<Time> for Time {
    type Output = u64;
    fn sub(self, rhs: Time) -> u64 {
        let a = self.secs_since_last_mid();
        let b = rhs.secs_since_last_mid();
        assert!(b >= a);
        b - a
    }
}
impl ops::Add<Time> for Date {
    type Output = Timestamp;
    fn add(self, time: Time) -> Timestamp {
        Timestamp::from_naive(
            self.0
            .checked_add_signed(chrono::TimeDelta::try_days(time.offset as _).unwrap_or_default())
            .unwrap_or_default()
            .and_hms_opt(time.h as u32, time.m as u32, time.s as u32)
            .unwrap_or_default()
        )
    }
}

fn _add(d: Date, t: Time) {
    let ts = d + t;
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use chrono::Datelike;
        write!(f, "{}-{}-{}", self.0.year(), self.0.month(), self.0.day())
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:02}:{:02}:{:02}", self.h, self.m, self.s)?;
        match self.offset {
            o if o>0 => write!(f, " +{o}"),
            o if o<0 => write!(f, " {o}"),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Time;
    #[test]
    fn parse_origin_time() {
        let p = Time::from_trip_origin;
        let t = Time::new_with_offset;
        assert_eq!(p("021150").unwrap(), t(3, 31, 30, 0));
        assert_eq!(p("-0000200").unwrap(), t(23, 58, 0, -1));
        assert_eq!(p("00145000").unwrap(), t(0, 10, 0, 1));
        assert_eq!(p("134200").unwrap(), t(22, 22, 0, 0));
    }
}
