
use reqwest;
use std::{path::Path, str::FromStr, marker::PhantomData, fmt::Display, any::type_name};
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use crate::{Timestamp, msg::{Route, StopId}};
use tokio::fs;
use anyhow::Context as _;
use tracing::{debug};
use reqwest::Url;

const PREFER_CACHE: bool = false;

pub struct Client(reqwest::Client);

impl Default for Client {
    fn default() -> Self {
        Client(reqwest::Client::default())
    }
}

impl Client {
    pub async fn get_equipment(&self) -> anyhow::Result<Vec<AccessEquipment>> {
        let url = "https://api-endpoint.mta.info/Dataservice/mtagtfsfeeds/nyct%2Fnyct_ene_equipments.json";
        self.get_inner(url, Path::new("cache/equipment.json")).await
    }
    pub async fn get_outage(&self) -> anyhow::Result<Vec<AccessOutage>> {
        let url = "https://api-endpoint.mta.info/Dataservice/mtagtfsfeeds/nyct%2Fnyct_ene.json";
        self.get_inner(url, Path::new("cache/outages.json")).await
    }
    pub async fn get_outages_nocache(&self) -> anyhow::Result<Vec<AccessOutage>> {
        let url = "https://api-endpoint.mta.info/Dataservice/mtagtfsfeeds/nyct%2Fnyct_ene.json";
        self.get_inner(url, Path::new("/dev/null")).await
    }

    pub async fn get_complexes(&self) -> anyhow::Result<Vec<ComplexInfo>> {
        let url = "https://data.ny.gov/resource/5f5g-n3cz.json";
        self.get_inner(url, Path::new("cache/complexes.json")).await
    }
    pub async fn get_entrances(&self) -> anyhow::Result<Vec<SubwayEntrance>> {
        let base = "https://data.ny.gov/resource/i9wp-a4ja.json";
        let mut ret = vec![];
        for i in 0..10 {
            let mut params = vec![("$limit", "1000".to_string())];
            if i > 0 {
                params.push(("$offset", (i*1000).to_string()));
            }
            let url = Url::parse_with_params(base, &params)?;
            let path = format!("cache/entrances.{i}.json");
            let mut new: Vec<_> = self.get_inner(url.as_str(), Path::new(&path)).await?;
            let len = new.len();
            ret.append(&mut new);
            if len < 1000 { break }
        }
        Ok(ret)
    }

    async fn get_inner<T: DeserializeOwned>(&self, url: &str, path: &Path) -> anyhow::Result<T> {
        let body = if PREFER_CACHE && path.exists() {
            debug!("Loading {} from DISK", type_name::<T>());
            fs::read_to_string(path)
                .await
                .with_context(|| format!("Failed to load file {}", path.display()))?
        } else {
            debug!("Fetching {} from mta.info", type_name::<T>());
            let rsp = self.0.get(url)
                .send().await
                .with_context(|| format!( "Failed to request {url}"))?;
            let body = rsp.text().await.context("Failed to accumulate")?;
            if let Err(e) = fs::write(path, &body).await {
                tracing::warn!("Failed to write response to disk {path:?}: {e}");
            }
            body
        };
        Ok(serde_json::from_str(&body)?)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccessEquipment {
    pub station: String,
    // borough: String,
    #[serde(rename = "trainno", deserialize_with = "parse_route_list")]
    pub trains: Vec<Route>,
    pub equipmentno: EquipmentId,
    pub equipmenttype: String, // EL or ES
    pub serving: String,
    #[serde(rename = "ADA", deserialize_with = "parse_bool")]
    pub ada: bool,
    #[serde(deserialize_with = "parse_bool")]
    pub isactive: bool,
    #[serde(rename = "nonNYCT", deserialize_with = "parse_bool")]
    pub non_nyct: bool,
    pub shortdescription: String,
    #[serde(deserialize_with = "parse_route_list")]
    pub linesservedbyelevator: Vec<Route>,
    #[serde(rename = "elevatorsgtfsstopid", deserialize_with = "parse_stop_list")]
    pub stop_ids: Vec<StopId>, // hyphen-concatenated StopIds,
    // elevatorsgtfsstopid: Vec<String>, // hyphen-concatenated StopIds,
    pub elevatormrn: String, // slash-delimited numbers ?
    #[serde(rename = "stationcomplexid", deserialize_with = "parse_quoted_complex_id")]
    pub complex_id: ComplexId, // station id
    pub nextadanorth: String,
    pub nextadasouth: String,
    pub redundant: i32,
    pub busconnections: String, // e.g. "Bx6, Bx6 SBS, Bx13"
    pub alternativeroute: String,
}

/* example
{
	"0": {
		"station": "1 Av",
		"borough": "",
		"trainno": "L",
		"equipmentno": "EL293",
		"equipmenttype": "EL",
		"serving": "E 14 St and Avenue A (SW corner) to Canarsie-bound platform",
		"ADA": "Y",
		"isactive": "Y",
		"nonNYCT": "N",
		"shortdescription": "Street to Brooklyn-bound platform",
		"linesservedbyelevator": "L",
		"elevatorsgtfsstopid": "L06",
		"elevatormrn": "119",
		"stationcomplexid": "119",
		"nextadanorth": "117, L",
		"nextadasouth": "120, L",
		"redundant": 0,
		"busconnections": "M15, M15 SBS, M14A SBS, M14D SBS",
		"alternativeroute": "If you are on the street: Take a westbound M14A SBS or M14D SBS (E 14 St and 1 Av) to 4 Av for Union Sq-14 St station. Then use elevators for Canarsie-bound L service. If you are on the platform: Take a Canarsie-bound L to Bedford Av. Then transfer across the platform to an 8 Av-bound L to 1 Av."
	}
}

AccessOutage { "station" : "74 St-Broadway", 
 "borough" : "", 
 "trainno" : "E/F/M/R/7", 
 "equipment" : "ES451", 
 "equipmenttype" : "ES", 
 "serving" : "E/F/M/R lower mezzanine to underpass for 7 service", 
 "ADA" : "N", 
 "outagedate" : "06/26/2023 09:35:00 AM", 
 "estimatedreturntoservice" : "12/31/2024 11:45:00 PM", 
 "reason" : "Capital Replacement", 
 "isupcomingoutage" : "N",
 "ismaintenanceoutage" : "N" }

Subway Entrance {
"division":"IND",
"line":"Second Av",
"borough":"M",
"stop_name":"96 St",
"complex_id":"475",
"constituent_station_name":"96 St",
"station_id":"475",
"gtfs_stop_id":"Q05",
"daytime_routes":"Q",
"entrance_type":"Stair/Escalator",
"entry_allowed":"YES",
"exit_allowed":"YES",
"entrance_latitude":"40.7841841",
"entrance_longitude":"-73.9474",
"entrance_georeference":{"type":"Point", "coordinates":[-73.9474,40.7841841]}},
*/

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct SubwayEntrance {
    pub division: String,
    pub line: String,
    pub borough: String, // brooklyn == "B", bronx == "Bx", and SIR = S
    pub stop_name: String,
    #[serde(deserialize_with = "parse_quoted_complex_id")]
    pub complex_id: ComplexId,
    pub constituent_station_name: String,
    pub station_id: String,
    #[serde(rename = "gtfs_stop_id", deserialize_with = "parse_stop_list2")]
    pub stop_ids: Vec<StopId>,   // "; ".join(StopIds)
    #[serde(rename = "daytime_routes", deserialize_with = "parse_route_list2")]
    pub routes: Vec<Route>,
    pub entrance_type: String,
    #[serde(deserialize_with = "parse_bool")]
    pub entry_allowed: bool,
    #[serde(deserialize_with = "parse_bool")]
    pub exit_allowed: bool,
    #[serde(deserialize_with = "parse_quoted_float")]
    pub entrance_latitude: f64,
    #[serde(deserialize_with = "parse_quoted_float")]
    pub entrance_longitude: f64,
    // entrance_georeference
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct AccessOutage {
    pub station: String,
    // pub borough: String, // empty
    #[serde(rename = "trainno", deserialize_with = "parse_route_list")]
    pub routes: Vec<Route>,    // train list, incl LIRR
    pub equipment: EquipmentId,
    pub equipmenttype: String,
    pub serving: String,
    #[serde(rename = "ADA", deserialize_with = "parse_bool")]
    pub ada: bool,
    #[serde(deserialize_with = "parse_date")]
    pub outagedate: Datetime, // "%M/%D/%Y %H:%m%s PM"
    #[serde(deserialize_with = "parse_date")]
    pub estimatedreturntoservice: Datetime,
    pub reason: String,
    #[serde(deserialize_with = "parse_bool")]
    pub isupcomingoutage: bool,
    #[serde(deserialize_with = "parse_bool")]
    pub ismaintenanceoutage: bool,

    #[serde(default = "Timestamp::now")]
    pub asof: Timestamp,
}



#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ComplexInfo {
    #[serde(deserialize_with = "parse_quoted_complex_id")]
    pub complex_id: ComplexId,
    #[serde(deserialize_with = "parse_bool")]
    pub is_complex: bool,
    // #[serde(deserialize_with = "parse_quoted_float
    pub number_of_stations_in_complex: String,
    pub stop_name: String,
    pub display_name: String,   // stop_name w/ routes
    pub constituent_station_names: String,
    #[serde(rename = "gtfs_stop_ids", deserialize_with = "parse_stop_list3")]
    pub stop_ids: Vec<StopId>,
    pub borough: String,
    #[serde(deserialize_with = "parse_bool")]
    pub cbd: bool,
    #[serde(rename = "daytime_routes", deserialize_with = "parse_route_list2")]
    pub routes: Vec<Route>,
    pub structure_type: String, // e.g. Subway
    #[serde(deserialize_with = "parse_quoted_float")]
    pub latitude: f64,
    #[serde(deserialize_with = "parse_quoted_float")]
    pub longitude: f64,
    #[serde(deserialize_with = "parse_ada")]
    pub ada: AdaStatus,
    pub ada_notes: Option<String>,
}

fn sort_routes(rs: &str) -> String {
    let mut sorted: Vec<char> = rs.chars().collect();
    sorted.sort();
    sorted.dedup();
    sorted.into_iter().filter(|c| c.is_alphanumeric()).collect()
}

impl<'a> PartialEq<AccessOutage> for &'a ComplexInfo {
    fn eq(&self, out: &AccessOutage) -> bool {
        let name = out.station.replace("42St/Port Authority-", "42 St-Port Authority ");
        todo!() /*
        self.constituent_station_names.split("; ").find(|s| s == &name).is_some()
            && sort_routes(&self.daytime_routes) == sort_routes(&out.serving) */
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct ComplexId(u32);

crate::newt! {
    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, Deserialize, Serialize)]
    pub struct EquipmentId[6];
    // ES258X
}

use serde::{Deserializer, de};
use std::fmt;
use chrono::{FixedOffset, NaiveDateTime};
type Datetime = chrono::DateTime<chrono::FixedOffset>;

#[derive(Serialize, Debug, Clone, Copy)]
pub enum AdaStatus {
    No = 0,
    Full = 1,
    Partial = 2,
}

fn parse_date<'de, D: Deserializer<'de>>(deser: D) -> Result<Datetime, D::Error> {
    deser.deserialize_str(DatetimeVisitor)
}

fn parse_ada<'de, D: Deserializer<'de>>(deser: D) -> Result<AdaStatus, D::Error> {
    deser.deserialize_str(AdaVisitor)
}

fn parse_bool<'de, D: Deserializer<'de>>(deser: D) -> Result<bool, D::Error> {
    deser.deserialize_str(BoolVisitor)
}

fn parse_quoted_float<'de, D: Deserializer<'de>>(deser: D) -> Result<f64, D::Error> {
    deser.deserialize_str(QuotedVisitor::default())
}

fn parse_quoted_complex_id<'de, D: Deserializer<'de>>(deser: D) -> Result<ComplexId, D::Error> {
    deser.deserialize_str(QuotedVisitor::default()).map(ComplexId)
}

fn parse_stop_list<'de, D: Deserializer<'de>>(deser: D) -> Result<Vec<StopId>, D::Error> {
    deser.deserialize_str(List::from("/"))
}

fn parse_stop_list2<'de, D: Deserializer<'de>>(deser: D) -> Result<Vec<StopId>, D::Error> {
    deser.deserialize_str(List::from(" "))
}

fn parse_stop_list3<'de, D: Deserializer<'de>>(deser: D) -> Result<Vec<StopId>, D::Error> {
    deser.deserialize_str(List::from("; "))
}

fn parse_route_list<'de, D: Deserializer<'de>>(deser: D) -> Result<Vec<Route>, D::Error> {
    // hack to remove LIRR (irrelevant and too long)
    let strings = deser.deserialize_str(List::<String>::from("/"))?;
    Ok(strings.iter()
        .filter(|s| "LIRR" != s.as_str() && "METRO-NORTH" != s.as_str())
        .map(|s| s.parse::<Route>().unwrap())
        .collect())
}

fn parse_route_list2<'de, D: Deserializer<'de>>(deser: D) -> Result<Vec<Route>, D::Error> {
    deser.deserialize_str(List::<Route>::from(" "))
}

struct List<T>(&'static str, PhantomData<T>);

impl<T> From<&'static str> for List<T> {
    fn from(delim: &'static str) -> Self {
        List(delim, PhantomData)
    }
}

impl<'de, T: FromStr> de::Visitor<'de> for List<T>
    where T::Err: Display
{
    type Value = Vec<T>;
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "'{}'-delimited list of {}", self.0, type_name::<T>())
    }
    fn visit_str<E: de::Error>(self, s: &str) -> Result<Self::Value, E> {
        let mut elems: Vec<T> = vec![];
        for (i, chunk) in s.split(self.0).enumerate() {
            match chunk.parse() {
                Ok(x) => elems.push(x),
                Err(e) => return Err(de::Error::custom(format!(
                            "Elem {i} of '{s}' invalid {}: {e}", type_name::<T>())))
            }
        }
        Ok(elems)
    }
}

struct DatetimeVisitor;

impl<'de> de::Visitor<'de> for DatetimeVisitor {
    type Value = Datetime;
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("datetime formatted")
    }
    // eg "06/26/2023 09:35:00 AM"
    fn visit_str<E: de::Error>(self, s: &str) -> Result<Self::Value, E> {
        let fmt = "%m/%d/%Y %I:%M:%S %p";
        let tz = FixedOffset::east_opt(5 * 60 * 60).unwrap();
        match NaiveDateTime::parse_from_str(s, fmt) {
            Ok(dt) => Ok(dt.and_local_timezone(tz).unwrap()),
            Err(e) => Err(de::Error::custom(e)),
        }
    }
}

struct AdaVisitor;

impl<'de> de::Visitor<'de> for AdaVisitor {
    type Value = AdaStatus;
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("ADA status: 0-2")
    }
    fn visit_str<E: de::Error>(self, s: &str) -> Result<Self::Value, E> {
        match s {
            "0" => Ok(AdaStatus::No),
            "1" => Ok(AdaStatus::Full),
            "2" => Ok(AdaStatus::Partial),
            _ => Err(de::Error::custom(format!("ADA status '{s}' out of bounds"))),
        }
    }
}

struct BoolVisitor;

impl<'de> de::Visitor<'de> for BoolVisitor {
    type Value = bool;
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("'Y' or 'N'")
    }
    fn visit_str<E: de::Error>(self, s: &str) -> Result<bool, E> {
        match s {
            "T" | "TRUE"  | "Y" | "YES" => Ok(true),
            "F" | "FALSE" | "N" | "NO" => Ok(false),
            _ => Err(de::Error::custom(format!("Unexpected bool str '{s}'"))),
        }
    }
    fn visit_u32<E: de::Error>(self, x: u32) -> Result<bool, E> {
        match x {
            0 => Ok(false),
            1 => Ok(true),
            __ => Err(de::Error::custom(format!("bool should be 0 or 1, not {x}"))),
        }
    }
}

#[derive(Default)]
struct QuotedVisitor<T>(PhantomData<T>);
impl<'de, T: FromStr> de::Visitor<'de> for QuotedVisitor<T>
    where T::Err: Display
{
    type Value = T;
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Quoted {}", type_name::<T>())
    }
    fn visit_str<E: de::Error>(self, s: &str) -> Result<T, E> {
        // if ! s.starts_with('"') {
        //     return Err(de::Error::custom(format!("{} should start with a quote, '{s}'", type_name::<T>())));
        // }
        // if ! s.ends_with('"') {
        //     return Err(de::Error::custom(format!("{} should end with a quote, '{s}'", type_name::<T>())));
        // }
        let inner = &s; // &s[1..s.len()-1];
        match inner.parse() {
            Ok(t) => Ok(t),
            Err(e) => Err(de::Error::custom(format!("{} {e}: {s}", type_name::<T>()))),
        }
    }
}

impl fmt::Display for ComplexId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
