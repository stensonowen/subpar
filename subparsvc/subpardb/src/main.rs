
use subpar::{ Client, Feed, Listener, msg::Update, };
use subpardb::Db;
use tokio_stream::StreamExt as _;
use std::time::Duration;
use tracing::{error, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "warn");
    tracing_subscriber::fmt::init();

    let db = Db::new(
        "postgres".to_string(), // user
        "sergtsop".to_string(), // pass
        "localhost".to_string())?;
    db.reset_all().await?;
    
    let feeds = vec![
        Feed::from_static("l"),
        Feed::from_static("1234567"),
        Feed::from_static("ace"),
        Feed::from_static("bdfm"),
        Feed::from_static("g"),
        Feed::from_static("jz"),
        Feed::from_static("nqrw"),
        Feed::from_static("si"),
    ];

    let interval = Duration::new(3, 0);
    let mut stream = Listener::new(Client::default(), feeds, interval).spawn();
    while let Some(rsp) = stream.next().await {
        let Some(rsp_uuid) = db.responses.upsert(&rsp).await? else {
            info!("duplicate response");
            continue
        };
        let (mut pos, mut sch, mut spls, mut alr, mut err) = (0, 0, 0, 0, 0);
        for elem in rsp.data.msgs {
            match elem {
                Ok(Update::Position(p)) => {
                    pos += 1;
                    let table = db.positions.clone();
                    tokio::spawn(async move {
                        let id = table.insert(rsp_uuid, &p).await.unwrap();
                        info!("inserted {id:?}");
                    });
                }
                Ok(Update::Schedule(s)) => {
                    sch += 1;
                    spls += s.stops().len();
                    let (t1, t2) = (db.schedules.clone(), db.stopplans.clone());
                    tokio::spawn(async move {
                        let id = t1.insert(rsp_uuid, &s).await.unwrap();
                        t2.insert_all(&id, &s).await.unwrap();
                        info!("inserted {id:?} and {} planned stops", s.stops().len());
                    });
                },
                Ok(Update::Alert) => {
                    // todo alerts table
                    alr += 1;
                    info!("alert");
                },
                Err(e) => {
                    err += 1;
                    // todo errors table
                    error!("error {e}");
                },
            }
        }
        println!("{}: Inserted {pos:>3} positions, {sch:>3} schedules, {spls:>4} planned stops. Skipped {alr} alerts and {err} errors", rsp.feed);
    }
    Ok(())
}


