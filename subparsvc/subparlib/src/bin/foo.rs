#![allow(unused)]

use subpar::{api, Client, Feed, ManifestStops, /* PollClient*/};
use std::collections::HashMap;
use tracing::info;

fn is_match(e: &api::SubwayEntrance, o: &api::AccessOutage) -> bool {
    #[derive(Eq, PartialEq, Debug)]
    struct Key {
        name: String,
    }
    impl From<&api::SubwayEntrance> for Key {
        fn from(e: &api::SubwayEntrance) -> Self {
            Key { name: e.stop_name.clone() }
        }
    }
    impl From<&api::AccessOutage> for Key {
        fn from(o: &api::AccessOutage) -> Self {
            Key { name: o.station.clone() }
        }
    }
    false
}

// AccessEquipment.equipmentno matches AccessOutage.equipment
// AccessEquipment.elevatorsgtfsstopid '/'.join(parent_stop_ids)

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "info");
    tracing_subscriber::fmt::init();

    // let client = PollClient::with_cache("../cache/");
    let client = api::Client::default();

    // println!("equipment");
    // let equipment = client.get_equipment().await?;
    // println!("entrances");
    // let entrances = client.get_entrances().await?;
    // println!("outages");
    // let outages = client.get_outage().await?;
    // info!("{:?}", &equipment[0]);
    // return Ok(());
    // info!("{:?}", &complexes[0]);

    let stops = ManifestStops::from_file("archive/stops.txt");
    let ids: HashMap<_, _> = stops.iter().map(|r| (r.stop, r)).collect();

    let complexes = client.get_complexes().await?;
    for cplx in complexes {
        if cplx.stop_ids.len() == 1 { continue }
        println!("Complex '{}' ", cplx.display_name);
        for stop in cplx.stop_ids {
            println!("\t{stop}: {}", ids[&stop].name);
        }

    }


    Ok(())
}
