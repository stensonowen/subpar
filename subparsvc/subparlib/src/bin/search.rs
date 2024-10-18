use subpar::{ManifestStops, Client, Feed, FromGtfs as _};
use subpar::msg::{Route, Batch, Update, StopId};
use tracing::{debug, info};

fn route_to_feed(route: Route) -> Feed {
    match route.as_ref().to_lowercase().as_str() {
        "1" | "2" | "3" | "4" | "5" | "6" | "7" => Feed::from_static("1234567"),
        "a" | "c" | "e" => Feed::from_static("ace"),
        "b" | "d" | "f" | "m" => Feed::from_static("bdfm"),
        "g" => Feed::from_static("g"),
        "j" | "z" => Feed::from_static("jz"),
        "n" | "q" | "r" | "w" => Feed::from_static("nqrw"),
        "l" => Feed::from_static("l"),
        r => panic!("unsupported route {r}")
    }
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let mut args = std::env::args();
    let usage = "./a.out route stop [stops.txt] (e.g. ./a.out 6 638N)";
    let route: Route = args.nth(1).expect(usage).parse()?;
    let stop: StopId = args.next().expect(usage).parse()?;
    info!("args: ./a.out rt={route} s={stop}");
    // if let Some(path) = args.next() {
    if let Some(path) = Some("./archive/stops.txt") {
        info!("loading stops.txt from {path}");
        let stops = ManifestStops::from_file(&path);
        let stoprow = &stops[stop];
        println!("Info for stop {stop} (\"{}\") at {:?}", stoprow.name, stoprow.location);
    }

    let client = Client::default();
    let feed = route_to_feed(route.clone());
    debug!("Requesting {}", feed.name());
    let resp = client.fetch(feed.url()).await?;
    let data = protobuf::Message::parse_from_bytes(&resp)?;
    let batch = Batch::parse(&data)?;
    debug!("Response with {} messages", batch.msgs.len());
    let mut msgs = vec![];
    for msg in &batch.msgs {
        if let Ok(Update::Schedule(sched)) = msg {
            if sched.trip().route() == route {
                for s in sched.stops() {
                    if s.id == stop {
                        // println!("{} at {}", sched.trip(), s.times);
                        msgs.push((sched.trip(), s));
                    }
                }
            }
        }
    }
    msgs.sort_by_key(|(_,  s)| s.times.t0());
    for (t, s) in msgs {
        println!("\t{t} stops at {}", s.times);
    }

    Ok(())
}

