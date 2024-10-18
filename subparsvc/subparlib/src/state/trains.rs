
use crate::{Timestamp, api::{self, ComplexId}, msg::{self, StopId, TripIdStr}, client::Response};
use std::{time::Duration, sync::{Arc, Mutex, }, collections::HashMap};
use tracing::{info, warn};

type StopIds = HashMap< StopId, ComplexId >;
type UpcomingMsgsMap = HashMap< TripIdStr, Upcoming >;
type ByComplex<T> = HashMap< ComplexId, T >;

#[derive(serde::Serialize, Clone, Debug)]
pub struct Upcoming {
    trip: TripIdStr,
    stop: StopId,
    arrival: Timestamp,
    message: Timestamp,
}

#[derive(Clone)]
pub struct TrainStates {
    stops: Arc<StopIds>,
    trains: Arc<Mutex< ByComplex< UpcomingMsgsMap >>>,
}

impl TrainStates {
    pub fn new(cplxs: &[api::ComplexInfo]) -> Self {
        let mut stops: StopIds = HashMap::new();
        for cplx in cplxs {
            for &stop_id in &cplx.stop_ids {
                stops.insert(stop_id, cplx.complex_id);
            }
        }
        let trains = Arc::new(Mutex::new(HashMap::default()));
        TrainStates { stops: Arc::new(stops), trains }
    }
    pub fn update(&self, rsp: &Response) {
        let new = self.preprocess_rsp(rsp);
        let mut inner = self.trains.lock().unwrap();
        merge(&mut inner, &new);
    }
    pub fn get(&self, id: ComplexId) -> Option< Vec<Upcoming>  > {
        let mut elems: Vec<Upcoming> = {
            let lock = self.trains.lock().unwrap();
            lock.get(&id)?.values().cloned().collect()
        };
        elems.sort_by_key(|u| u.arrival);
        Some(elems)
    }
    fn preprocess_rsp(&self, rsp: &Response) -> ByComplex< UpcomingMsgsMap > {
        let message = rsp.data.time;
        let mut map: ByComplex<_> = self.stops.values()
            .map(|&c| (c, UpcomingMsgsMap::new()))
            .collect();
        for elem in &rsp.data.msgs {
            if let Ok(msg::Update::Schedule(s)) = elem {
                let trip = s.trip().name();
                for stopplan in s.stops() {
                    let stop = stopplan.id.parent();
                    let arrival = *stopplan.times.t0();
                    let u = Upcoming { trip, stop, message, arrival };
                    let Some(complex) = self.stops.get(&stop) else {
                        warn!("msg had unknown stop_id {stop}");
                        continue
                    };
                    map.entry(*complex).or_default().insert( trip, u );
                }
            }
        }
        map
    }
}

fn merge(lhs: &mut ByComplex< UpcomingMsgsMap >, rhs: &ByComplex< UpcomingMsgsMap > ) {
    for (cplx, msgs) in rhs {
        match lhs.get_mut(cplx) {
            None => { lhs.insert(*cplx, msgs.clone()); },
            Some(ref mut old) => { 
                for (trip, msg) in msgs {
                    match old.get_mut(&trip) {
                        None => { old.insert(*trip, msg.clone()); },
                        Some(ref mut slot) if slot.message <= msg.message => {
                            if slot.stop == msg.stop {
                                slot.message = msg.message;
                                slot.arrival = msg.arrival;
                            } else {
                                warn!("stop mismatch; {slot:?} {msg:?}");
                            }
                        },
                        Some(_) => {
                            warn!("weird; old msg time older than new msg time");
                        },
                    };
                }
                let cutoff = Timestamp::now() - Duration::from_secs(45);
                old.retain(|k, v| {
                    v.message > cutoff
                });
            }
        }
    }
}

