
use crate::{api::{self, EquipmentId, ComplexId}, msg::{Route}};
use chrono::{DateTime, FixedOffset};
use serde::Serialize;
use std::{sync::{Arc, RwLock}, collections::{HashSet, HashMap}};

type Complexes = HashMap< EquipmentId, ComplexId >;
type Elevators = HashMap< ComplexId, Vec< Elevator > >;

#[derive(Clone)]
pub struct ElevatorStates {
    elevators: Arc<RwLock< Elevators >>,
    complexes: Arc< Complexes >,
    summary_cache: Arc<RwLock<Option<ElevatorSummary>>>,
}

#[derive(Clone, Serialize)]
pub struct ElevatorSummary {
    outages: Vec<ComplexId>,
}

impl ElevatorStates {
    pub fn new(equipment: &[api::AccessEquipment]) -> Self {
        let mut els = Elevators::default();
        for eq in equipment {
            els.entry(eq.complex_id).or_default().push(eq.into());
        }
        let elevators = Arc::new(RwLock::new(els));
        let complexes = equipment.iter().map(|e| (e.equipmentno, e.complex_id)).collect();
        let complexes = Arc::new(complexes);
        let summary_cache = Arc::new(RwLock::new(None));
        ElevatorStates { elevators, complexes, summary_cache }
    }
    pub fn with_outages(self, outages: &[api::AccessOutage]) -> Self {
        self.update(outages);
        self
    }
    pub fn update(&self, outages: &[api::AccessOutage]) {
        *self.summary_cache.write().unwrap() = None;
        let mut map = self.elevators.write().unwrap();
        for els in map.values_mut() {
            for el in els {
                el.outage = None;
            }
        }
        for update in outages {
            let equip_id = &update.equipment;
            let Some(cplx) = self.complexes.get(equip_id) else {
                println!("equipment id {equip_id} not found");
                continue
            };
            let Some(els) = map.get_mut(cplx) else {
                println!("complex id {cplx} not found");
                continue
            };
            let Some(el) = els.iter_mut().find(|e| &e.id == equip_id) else {
                println!("(complex, equip) ({cplx}, {equip_id}) not found");
                continue
            };
            el.outage = Some(update.into());
        }
    }
    pub fn get(&self, id: ComplexId) -> Option<Vec<Elevator>> {
        let map = self.elevators.read().unwrap();
        map.get(&id).cloned()
    }
    pub fn get_summary(&self) -> ElevatorSummary {
        if let Some(s) = &*self.summary_cache.read().unwrap() {
            return s.clone();
        }
        let s = self.make_summary();
        *self.summary_cache.write().unwrap() = Some(s.clone());
        s
    }
    fn make_summary(&self) -> ElevatorSummary {
        let mut ids: HashSet<ComplexId> = HashSet::new();
        {
            let els = self.elevators.read().unwrap();
            for e in els.values().flatten() {
                if let Some(_) = &e.outage {
                    ids.insert(e.complex_id);
                }
            }
        }
        let outages = ids.into_iter().collect();
        ElevatorSummary { outages }
    }
}

#[derive(Serialize, Clone)]
pub struct Elevator {
    id: EquipmentId,
    complex_id: ComplexId,
    lines: Vec<Route>,
    is_escalator: bool,
    ada: bool,
    is_active: bool,
    desc: String,
    serving: String,
    nearby: Vec<(ComplexId, Route)>,
    buses: String,
    alt_desc: String,
    outage: Option<Outage>,
}


#[derive(Serialize, Clone)]
pub struct Outage {
    id: EquipmentId,
    start: DateTime<FixedOffset>,
    ada: bool,
    est_return: DateTime<FixedOffset>,
    reason: String,
    upcoming: bool,
    maintenance: bool,
}

impl From<&api::AccessOutage> for Outage {
    fn from(x: &api::AccessOutage) -> Self {
        Outage {
            id: x.equipment,
            start: x.outagedate,
            ada: x.ada,
            est_return: x.estimatedreturntoservice,
            reason: x.reason.clone(),
            upcoming: x.isupcomingoutage,
            maintenance: x.ismaintenanceoutage,
        }
    }
}

impl From<&api::AccessEquipment> for Elevator {
    fn from(x: &api::AccessEquipment) -> Self {
        Elevator {
            id: x.equipmentno,
            complex_id: x.complex_id,
            is_escalator: x.equipmenttype == "ES",
            is_active: x.isactive,
            ada: x.ada,
            serving: x.serving.clone(),
            lines: x.linesservedbyelevator.clone(),
            desc: x.shortdescription.clone(),
            nearby: vec![],
            buses: x.busconnections.clone(),
            alt_desc: x.alternativeroute.clone(),
            outage: None,
        }
    }
}
