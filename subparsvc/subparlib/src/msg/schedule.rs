
use crate::{Timestamp, msg::{TripId, StopId}};
use std::fmt;

#[derive(Debug, Clone)]
pub struct Schedule {
    trip: TripId,
    stops: Vec<StopPlan>,
    asof: Timestamp,
}

impl Schedule {
    pub fn new(trip: TripId, asof: Timestamp, stops: Vec<StopPlan>) -> Self {
        Schedule { trip, stops, asof }
    }
    pub fn trip(&self) -> TripId {
        self.trip.clone()
    }
    pub fn stops(&self) -> &[StopPlan] {
        &self.stops
    }
    pub fn asof(&self) -> Timestamp {
        self.asof
    }
}


#[derive(Debug, Clone)]
pub struct StopPlan {
    pub times: Times,
    pub id: StopId,
}

#[derive(Debug, Clone)]
pub enum Times {
    Last { arr: Timestamp },
    First { dep: Timestamp },
    Mid { arr: Timestamp, dep: Timestamp },
}

impl Times {
    pub fn new(arr: Option<Timestamp>, dep: Option<Timestamp>) -> anyhow::Result<Self> {
        match (arr, dep) {
            (Some(a), Some(d)) => Ok(Times::Mid { arr: a, dep: d }),
            (Some(a), None) => Ok(Times::Last { arr: a }),
            (None, Some(d)) => Ok(Times::First { dep: d }),
            (None, None) => Err(anyhow::anyhow!("Invalid 'times' w/ no times")),
        }
    }
    pub fn t0(&self) -> &Timestamp {
        match self {
            Times::Last { arr } => arr,
            Times::First { dep } => dep,
            Times::Mid { arr, .. } => arr,
        }
    }
    pub fn arr(&self) -> Option<&Timestamp> {
        match self {
            Times::Last { arr } | Times::Mid { arr, .. } => Some(arr),
            Times::First { dep } => None,
        }
    }
    pub fn dep(&self) -> Option<&Timestamp> {
        match self {
            Times::First { dep } | Times::Mid { dep, .. } => Some(dep),
            Times::Last { arr } => None,
        }
    }
}

impl StopPlan {
    pub fn new(id: StopId, times: Times) -> Self {
        StopPlan { times, id }
    }
}

impl fmt::Display for Times {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Times::*;
        match self {
            Last { arr } => write!(f, "FINAL {arr}"),
            First { dep } => write!(f, "FIRST {dep}"),
            Mid { arr, dep } => {
                let dur = dep.seconds_since(arr);
                if dur == 0 {
                    assert!(arr == dep, "{arr} != {dep}, ds = {dur}");
                    write!(f, "{arr}")
                } else {
                    write!(f, "{arr} for {dur}s")
                }
            }
        }
    }
}

impl fmt::Display for Schedule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.trip.as_str())?;
        if self.asof != Timestamp::epoch() {
            write!(f, " asof {}s ago", Timestamp::now().seconds_since(&self.asof))?;
        }
        f.write_str(": ")?;
        for (i, StopPlan { times, id }) in self.stops.iter().enumerate() {
            if i != 0 { write!(f, " → ")?; }
            write!(f, "{id} {times}")?;
        }
        Ok(())
    }
}
