#![allow(unused)]

use anyhow::{anyhow, bail, Context as _};
use tokio::{time, sync::mpsc, signal::{ctrl_c, unix as signal}};
use tokio_stream::{wrappers::ReceiverStream, StreamExt as _};
use tracing::{span, event, Level};

use subpar::{Client, Feed, msg, Timestamp};
use std::{fmt, sync::Arc};



#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // simple_logger::init_with_level(log::Level::Debug).unwrap();

    std::env::set_var("RUST_LOG", "info");
    tracing_subscriber::fmt::init();

    // simple_logger::init_with_level(log::Level::Debug).unwrap();
    let feeds = vec![
        Feed::from_static("nqrw"),
        Feed::from_static("l"),
    ];

    let client = Client::default();
    let interval = std::time::Duration::new(4, 0);
    let mut stream = subpar::Listener::new(client, feeds, interval).spawn();
    while let Some(rsp) = stream.next().await {
        println!("{}", rsp);
        for elem in rsp.data.msgs.iter() {
            match elem {
                Ok(msg::Update::Position(p)) => {
                    // tracing::info!("{p}");
                },
                Ok(msg::Update::Schedule(s)) => {
                    tracing::info!("{s}");
                },
                Ok(_) => {},
                // Ok(x) => x,
                Err(e) => {
                    tracing::warn!("failed to parse feed msg update {e}");
                    continue
                }
            };

        }
    }


    Ok(())
}

