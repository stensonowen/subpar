use std::{ops, fmt, collections::HashMap};
use crate::msg::StopId;
use super::csv::{FileIter, FromCsv, CsvIter};

pub struct ManifestStops {
    stops: HashMap<StopId, StopRow>,
}

#[derive(Debug)]
pub struct StopRow {
    pub stop: StopId,
    pub name: String,
    pub parent: Option<StopId>,
    pub location: (f64, f64),
}

impl ManifestStops {
    pub fn from_file(path: &str) -> Self {
        let mut stops = HashMap::<StopId, StopRow>::new();
        for row in FileIter::<StopRow>::new(path) {
            tracing::debug!("parsed {row}");
            if let Some(dupe) = stops.get(&row.stop) {
                panic!("Duplicate Row: {dupe:?} vs {row:?}");
            }
            stops.insert(row.stop, row);
        }
        ManifestStops { stops }
    }
    pub fn len(&self) -> usize {
        self.stops.len()
    }
    pub fn iter(&self) -> impl Iterator<Item = &StopRow> {
        self.stops.values()
    }
}

impl ops::Index<StopId> for ManifestStops {
    type Output = StopRow;
    fn index(&self, id: StopId) -> &StopRow {
        tracing::debug!("Looking up stop {id}");
        &self.stops[&id]
    }
}

impl fmt::Display for StopRow {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (lat, lon) = self.location;
        write!(f, "{} \"{}\" @ ({}, {})", self.stop, self.name, lat, lon)?;
        if let Some(x) = self.parent {
            write!(f, " ({x})")?;
        }
        Ok(())
    }
}


impl FromCsv for StopRow {
    const HEADER: &'static str = concat!(
        "stop_id,stop_code,stop_name,stop_desc,",
        "stop_lat,stop_lon,zone_id,",
        "stop_url,location_type,parent_station"
    );
    const FILENAME: &'static str = "stops.txt";
    fn parse(mut row: CsvIter) -> Self {
        let [stop_id, _, stop_name, _] = row.next_n();
        let location: (f64, f64) = (row.next_as(), row.next_as());
        let [_, _, _, parent] = row.next_n();
        let stop: StopId = stop_id
            .parse()
            .unwrap_or_else(|e| panic!("Bad stop_id: {e:?}"));
        let parent = if parent == "" {
            None
        } else {
            let res: Result<StopId, _> = parent.parse();
            Some(res.unwrap_or_else(|e| panic!("Bad parent stop: {e:?}")))
        };
        row.finish();
        let name = stop_name.to_owned();
        StopRow {
            stop,
            name,
            location,
            parent,
        }
    }
}
