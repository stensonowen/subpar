use crate::{Timestamp, msg};


pub struct Batch {
    pub time: Timestamp,
    pub msgs: Vec<anyhow::Result<msg::Update>>,
}


