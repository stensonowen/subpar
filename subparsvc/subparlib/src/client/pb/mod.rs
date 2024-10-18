#![allow(unused_imports)]

/// gtfs-realtime protobuf types
// https://developers.google.com/transit/gtfs-realtime/gtfs-realtime.proto
pub mod gtfs_realtime;
pub use gtfs_realtime as gtfs;

/// nyct subway extension 1001 to gtfs-realtime
// https://api.mta.info/nyct-subway.proto.txt
pub(crate) mod nyct_subway;
pub(crate) use nyct_subway as nyct;

