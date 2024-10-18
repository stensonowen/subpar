
use std::{sync::{Arc}, collections::{HashMap}};
use crate::{msg::{StopId, Route, }, api::{self, ComplexId}};

type ComplexMap = HashMap<ComplexId, ComplexMeta>;

#[derive(Clone)]
pub struct ComplexStates {
    meta: Arc< ComplexMap >,
}

#[derive(serde::Serialize, Clone)]
pub struct ComplexMeta {
    name: String,
    ada: api::AdaStatus,
    ada_notes: Option<String>,
    coord: (f64, f64),
    routes: Vec<Route>,
    stops: Vec<StopId>,
    entrances: Vec<api::SubwayEntrance>,
}

impl ComplexStates {
    pub fn new(cplxs: &[api::ComplexInfo], entrs: &[api::SubwayEntrance]) -> Self {
        let mut meta: ComplexMap = cplxs.iter()
            .map(|c| (c.complex_id, c.into()))
            .collect();
        for entr in entrs {
            let id = entr.complex_id;
            match meta.get_mut(&id) {
                Some(m) => m.entrances.push(entr.clone()),
                None => println!("subway entrance w/ unknown complex id {id}"),
            }
        }
        let meta = Arc::new(meta);
        ComplexStates { meta }
    }
    pub fn get(&self, id: ComplexId) -> Option<ComplexMeta> {
        self.meta.get(&id).cloned()
    }
}

impl From<&api::ComplexInfo> for ComplexMeta {
    fn from(c: &api::ComplexInfo) -> Self {
        ComplexMeta {
            name: c.stop_name.clone(),
            ada: c.ada,
            ada_notes: c.ada_notes.clone(),
            coord: (c.latitude, c.longitude),
            routes: c.routes.clone(),
            stops: c.stop_ids.clone(),
            entrances: vec![],
        }
    }
}
