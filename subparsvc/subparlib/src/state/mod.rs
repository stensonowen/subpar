
use crate::api::{self, ComplexId};

pub mod complex;
pub use complex::{ComplexStates, ComplexMeta};

pub mod trains;
pub use trains::{ TrainStates, Upcoming };

pub mod elevators;
pub use elevators::{Elevator, ElevatorStates, ElevatorSummary};

// pub mod upcoming;


// single state supported by axum
#[derive(Clone)]
pub struct /*United*/ States {
    pub trains: TrainStates,
    pub elevators: ElevatorStates,
    pub complexes: ComplexStates,
}

impl States {
    pub fn new(
        complexes: &[api::ComplexInfo],
        elevators: &[api::AccessEquipment],
        e_outages: &[api::AccessOutage],
        entrances: &[api::SubwayEntrance],
    ) -> Self {
        States {
            trains: TrainStates::new(&complexes),
            elevators: ElevatorStates::new(elevators).with_outages(e_outages),
            complexes: ComplexStates::new(&complexes, &entrances),
        }
    }
    pub fn get_full(&self, id: ComplexId) -> Option<ComplexFull> {
        let meta = self.complexes.get(id);
        let elevators = self.elevators.get(id)?;
        let upcoming = self.trains.get(id)?;
        Some(ComplexFull { meta, upcoming, elevators })
    }
}

#[derive(serde::Serialize)]
pub struct ComplexFull {
    meta: Option<ComplexMeta>,
    upcoming: Vec<Upcoming>,
    elevators: Vec<Elevator>,
}

