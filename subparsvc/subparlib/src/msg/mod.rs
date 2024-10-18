use crate::newt;

mod schedule;
pub use schedule::{Schedule, StopPlan, Times};

mod position;
pub use position::{Position, PositionStatus};

mod batch;
pub use batch::{Batch};

mod datetime;
pub use datetime::{Date, Time};

mod trip;
pub use trip::{TripId, TripParts, TripDir};

#[derive(Debug, Clone)]
pub enum Update {
    Alert,
    Position(Position),
    Schedule(Schedule),
}

newt! {
    /// Route letter, excluding local/express-ness.
    /// Only shuttles use more than 1 character.
    /// e.g. '6' or 'SIR'.
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Route[3];
}

newt! {
    /// A Station (parent) or Platform (child) e.g. 101 or 101N
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct StopId[4];
}

newt! {
    /// e.g. '028650_7..N'
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct TripIdStr[20];
}

impl StopId {
    pub fn is_parent(&self) -> bool {
        let Some(tail) = self.0.as_str().chars().rev().next() else { return false };
        tail == 'N' || tail == 'S'
    }
    pub fn parent(&self) -> Self {
        if self.is_parent() {
            let mut parent = self.clone();
            parent.0.pop().unwrap();
            parent
        } else {
            self.clone()
        }
    }
}
