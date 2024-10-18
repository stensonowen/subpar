
use anyhow::anyhow;
use crate::{Timestamp, msg::{StopId, TripId, }};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionStatus {
    Nothing,
    At,
    Near,
    EnRoute,
}

impl TryFrom<u32> for PositionStatus {
    type Error = anyhow::Error;
    fn try_from(x: u32) -> anyhow::Result<Self> {
        use PositionStatus::*;
        match x {
            0 => Ok(Nothing),
            1 => Ok(At),
            2 => Ok(Near),
            3 => Ok(EnRoute),
            _ => Err(anyhow!("bad position status {x}")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Position {
    pub trip: TripId,
    // asof: Timestamp,
    pub time: Timestamp,
    pub stop: StopId,
    pub stop_n: Option<u32>,
    pub status: PositionStatus,
}



impl Position {
    pub fn new(
        trip: TripId,
        stop: StopId,
        stop_n: Option<u32>,
        status: PositionStatus,
        time: Timestamp,
    ) -> Self {
        Position {
            trip,
            stop,
            stop_n,
            status,
            time,
        }
    }
    pub fn trip(&self) -> TripId {
        self.trip.clone()
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Position { trip, time, stop, stop_n, status } = self;
        // write!(f, "{time} {trip} \t{stop:?} ({stop_n:?}) \t{status:?}")
        write!(f, "{time} {trip} \t{stop:?}")?;
        if let Some(n) = stop_n {
            write!(f, " #{}", *n)?;
        }
        if *status != PositionStatus::Nothing {
            write!(f, " '{:?}'", status)?;
        }
        Ok(())
    }
}

