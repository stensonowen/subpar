
use crate::msg::Batch;
use super::{Feed, Client, gtfs};
use crate::{Timestamp, proto::FromGtfs as _, msg::Update};
use uuid::Uuid;
use metrohash::MetroHash128;
use futures::future::join_all;

use std::{fmt, sync::Arc, time::Duration};

use tokio::{time, sync::mpsc};
use tokio_stream::{StreamExt as _, wrappers::ReceiverStream};
use tracing::{Level, span, event, warn};


pub struct Response {
    pub t_req: Timestamp,
    pub t_rsp: Timestamp,
    pub feed: Feed,
    pub data: Batch, 
    pub hash: Uuid,
    pub length: usize,
}

pub struct Listener {
    client: Arc<Client>,
    feeds: Vec<Feed>,
    sched: time::Interval,
}


impl Listener {

    pub fn new(
        client: Client,
        feeds: Vec<Feed>,
        interval: Duration,
    ) -> Self {
        let sched = time::interval(interval);
        let client = Arc::new(client);
        Listener { client, feeds, sched }
    }

    pub fn spawn(self) -> ReceiverStream<Response> {
        let Listener { client, feeds, mut sched } = self;
        let n = feeds.len();
        let (tx, rx) = mpsc::channel(n*2 + 1);
        tokio::spawn(async move {
            let parent = span!(Level::INFO, "listener", ?feeds);
            let _guard = parent.enter();
            loop {
                sched.tick().await;
                tracing::trace!("Woke up to poll {n} feeds");
                let send = |f: &Feed| Self::try_send(client.clone(), f.clone(), tx.clone());
                let all = join_all(feeds.iter().map(send));
                match time::timeout(sched.period(), all).await {
                    Ok(_) => (),
                    Err(_) => warn!("listener loop didn't complete in time"),
                }
                /* we'd prefer to run the fetches in a task so if one request is slow it doesn't
                 * delay the start of the next round (a slow feed can interfere with all fast feeds).
                 * However, some of these seem requests seem to take a long time (several minutes),
                 * and the task count gradually grows faster than they're cleaned up.
                 * Timeout handling in the client and increasing the channel capacity don't seem to
                 * help.
                 * Next I'll try putting the join_all in a timeout if this doesn't inhibit logging
                 * which requests never complete.
                 */
                // tokio::spawn(all);
            }
        });
        ReceiverStream::new(rx)
    }

    #[tracing::instrument(skip(cli, tx))]
    async fn try_send(cli: Arc<Client>, feed: Feed, tx: mpsc::Sender<Response>) {
        let t_req = Timestamp::now();
        let name = feed.name();
        let bytes = match cli.fetch2(feed.url()).await {
            Ok(bs) => bs.to_vec(), // clone alert
            Err(e) => { return tracing::warn!("Fetch failure for {name}: {e}") }
        };
        let t_rsp = Timestamp::now();
        let msgs: gtfs::FeedMessage = match protobuf::Message::parse_from_bytes(&bytes) {
            Ok(ms) => ms,
            Err(e) => { return tracing::error!("Parse failure for {name}: {e}") },
        };
        let batch = match Batch::parse(&msgs) {
            Ok(br) => br,
            Err(e) => {
                return tracing::warn!("Failed to parse results out of feed message: {e}");
            },
        };
        let resp = Response::new(batch, feed, &bytes, t_req, t_rsp);
        if tx.capacity() == 0 {
            warn!("channel capacity at 0");
        }
        match tx.send(resp).await {
            Ok(()) => event!(Level::DEBUG, "Submitted response"),
            Err(e) => tracing::warn!("Listener channel overflowed"),
        };
    }

}

impl Response {
    pub fn new(msgs: Batch, feed: Feed, data: &[u8], t_req: Timestamp, t_rsp: Timestamp) -> Self {
        Response {
            feed,
            t_req,
            t_rsp,
            data: msgs,
            length: data.len(),
            hash: hash(data),
        }
    }
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let [mut p, mut s, mut a, mut e] = [0;4];
        for elem in &self.data.msgs {
            match elem {
                Ok(Update::Position(_)) => p += 1,
                Ok(Update::Schedule(_)) => s += 1,
                Ok(Update::Alert) => a += 1,
                Err(_) => e += 1,
            }
        }
        write!(f, "[{:<7}  p={p} s={s} a={a} e={e}]", self.feed.name())
    }
}

fn hash(bytes: &[u8]) -> Uuid {
    use std::hash::Hasher as _;
    let mut hasher = MetroHash128::new();
    hasher.write(bytes);
    let (hi, lo) = hasher.finish128();
    Uuid::from_u64_pair(hi, lo)
}

