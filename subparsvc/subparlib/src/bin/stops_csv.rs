#![allow(unused)]
use anyhow::anyhow;
use tracing::info;
use subpar::{msg::StopId, manifest::ManifestStops};
use std::{env};


fn main() -> anyhow::Result<()> {
    env::set_var("RUST_LOG", "debug");
    tracing_subscriber::fmt::init();
    let mut args = env::args();
    let stops_file = args.nth(1).ok_or_else(|| anyhow!("usage: ./a stops.csv"))?;
    tracing::info!("Reading {}", stops_file);
    let stops = ManifestStops::from_file(&stops_file);
    // for stop in stops.iter() { println!("{:?}", stop); }
    Ok(())
}

