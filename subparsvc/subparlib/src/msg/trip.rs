// use super::types as t;
use super::{Date, Time, TripIdStr, Route};
use crate::Timestamp;
use anyhow::{anyhow, Context as _};
use std::{fmt, str};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TripId {
    text: TripIdStr,
    data: TripParts,
    day: Date,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TripParts {
    // FromStr
    pub rt: Route,
    pub dir: TripDir,
    pub time: Time,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TripDir {
    North, // also East
    South,
}

impl str::FromStr for TripParts {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> anyhow::Result<Self> {
        // do i really need a regular expression for this
        // nah
        let (d1, d2) = match (s.find('_'), s.find('.')) {
            (Some(i), Some(j)) if i < j => Ok((i, j)),
            (Some(_), Some(_)) => Err(anyhow!("trip parts out of order")),
            _ => Err(anyhow!("trip parts missing delimiter(s)")),
        }
        .with_context(|| format!("parse trip parts '{s}'"))?;
        let (origin, route, tail) = (&s[..d1], &s[d1 + 1..d2], &s[d2..]);
        if tail.len() > 4 {
            // println!("congrats it's a shape id i threw it away {tail}");
        }
        let dir = if tail.find(".N").is_some() {
            TripDir::North
        } else if tail.find(".S").is_some() {
            TripDir::South
        } else {
            anyhow::bail!("no clue what to make of this trip tail {tail}")
        };
        Ok(TripParts {
            rt: route.parse().context("trip parts")?,
            dir,
            time: Time::from_trip_origin(origin).context("trip origin")?,
        })
    }
}

impl TripId {
    pub fn parse(s: &str, day: Date) -> anyhow::Result<Self> {
        let data = s.parse().with_context(|| format!("tokenize {s}"))?;
        let text = s.parse().with_context(|| format!("copy trip_id {s}"))?;
        Ok(TripId { text, day, data })
    }
    pub fn name(&self) -> TripIdStr {
        self.text
    }
    pub fn as_str(&self) -> &str {
        self.text.as_ref()
    }
    pub fn departed(&self) -> Timestamp {
        self.day + self.data.time
    }
    pub fn data(&self) -> TripParts {
        self.data.clone()
    }
    pub fn route(&self) -> Route {
        self.data.rt.clone()
    }
    pub fn dir(&self) -> TripDir {
        self.data.dir.clone()
    }
    pub fn date(&self) -> chrono::NaiveDate {
        self.day.to_naive()
    }
    pub fn origin(&self) -> Timestamp {
        let t = self.data.time;
        let date = self.date() + chrono::TimeDelta::days(t.offset as _);
        let naive = date.and_hms_opt(t.h as _, t.m as _, t.s as _).unwrap();
        let dt = chrono::DateTime::from_naive_utc_and_offset(naive, chrono::Utc);
        // let dt = dt.with_utc()
        // let dt = dt.with_timezone(chrono::Local);
        Timestamp::from_utc(dt)
        
    }
}

impl Default for TripId {
    fn default() -> Self {
        TripId {
            text: "000000_0..N".parse().unwrap(),
            data: TripParts {
                rt: "0".parse().unwrap(),
                dir: TripDir::North,
                time: Time::new(0, 0, 0),
            },
            day: Date::make(2020, 1, 1),
        }
    }
}

impl fmt::Display for TripId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let p = &self.data;
        let day = self.day.to_naive();
        write!(f, "{}{} (", p.rt, p.dir)?;
        if day != chrono::Utc::now().date_naive() {
            write!(f, "{} ", day.format("%m-%d"))?;
        }
        write!(f, "{})", p.time)
    }
}

impl fmt::Display for TripDir {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TripDir::North => 'N',
                TripDir::South => 'S',
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{Time, TripDir, TripParts};

    #[test]
    fn tokenize_trip_parts() {
        fn tp(r: &str, d: char, t: u32) -> TripParts {
            let dir = match d {
                'N' => TripDir::North,
                'S' => TripDir::South,
                _ => panic!("invalid direction indicator {d}"),
            };
            let time = Time::from_trip_origin(&t.to_string()).unwrap();
            TripParts {
                rt: r.parse().unwrap(),
                dir,
                time,
            }
        }
        let p = |s: &str| s.parse::<TripParts>();
        assert_eq!(p("134200_L..N").unwrap(), tp("L", 'N', 134200));
        assert_eq!(p("134200_GS.S").unwrap(), tp("GS", 'S', 134200));
        assert_eq!(p("101200_GS.S04R").unwrap(), tp("GS", 'S', 101200));
    }
}
