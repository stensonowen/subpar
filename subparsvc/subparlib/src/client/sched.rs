
use tokio::time;
use chrono::{DurationRound as _};
use std::{fmt};

use super::{Client, Feed};
// use crate::{Trigger, Client, Feed, Writer};

const REPEAT_PERIOD: time::Duration = time::Duration::from_secs(30);


#[derive(Debug)]
pub(crate) struct Scheduler {
    feeds: Vec<Feed>,
    client: Client,
    // writer: Writer,
}

impl Scheduler {

    pub fn new(feeds: Vec<Feed>) -> Self {
        let client = Client::default();
        let writer = todo!();
        Scheduler { client, feeds }
    }

    async fn wait_until_start() {
        let now = chrono::Utc::now();
        let period = chrono::Duration::from_std(REPEAT_PERIOD).unwrap();
        let next_window = now.duration_trunc(period).unwrap() + period;
        let until_next = next_window - now;
        log::info!("Starting in {} ms", until_next.num_milliseconds());
        let until_next = until_next.to_std().unwrap();
        time::sleep(until_next).await;
    }

    async fn launch_one(&self) {
        // let trigger = todo!();
        // let data = match self.client.get(trigger.feed()).await {
        //     Some(x) => x,
        //     None => return,
        // };
        // let len = data.len();
        // let dur_ms = (chrono::Utc::now() - trigger.time).num_milliseconds();
        // log::debug!("{} took {}ms for {}", trigger, dur_ms, ByteSize(len));
        // if let Err(e) = self.writer.write(trigger, data).await {
        //     log::warn!("Failed to write {} bytes ({:?}): {}", len, trigger, e);
        // }
    }

    async fn launch_all(&self) {
        let now = chrono::Utc::now();
        let jobs: Vec<_> = self.feeds.iter().map(|f| {
            self.launch_one(Trigger::new(f, now))
        }).collect();
        // add a timeout on iter?
        futures::future::join_all(jobs).await;
    }

    pub async fn run(self) -> ! {
        Self::wait_until_start().await;
        let mut interval = time::interval(REPEAT_PERIOD);
        loop {
            interval.tick().await;
            log::debug!("- - - - - - - - - - - - - - - - - - - -");
            self.launch_all().await;
        }
    }

}


// // // // // // // //
// Timer
// // // // // // // //

struct Timer<F> {
    func: F,
}

impl<F: Fn()->()> Timer<F> {

    fn new(f: F) -> Self {
        Timer { func: f }
    }

    fn wait_until_start(&self) {
    }

    fn wait_until_next_tick(&self) {
    }

}

use std::future::Future;

#[derive(Debug, PartialEq, Eq)]
enum Event {
    Delay { ms: usize },
    Call,
    Payload(String),
    // Future(Box<dyn Future<Output=T>>),
}

trait Unit {
    const SIZE: usize = 1000;
    const DELIM: &'static str = "";
    const SUFFIXES: &'static[ &'static str ];
}

#[derive(Debug)]
struct ByteSize(pub usize);
impl Unit for ByteSize {
    const SIZE: usize = 1024;
    const SUFFIXES: &'static [ &'static str ] = &[
        "b", "kb", "mb", // should be caps
    ];
}
impl fmt::Display for ByteSize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (mut rem, mut commas) = (self.0, 0);
        while rem >= Self::SIZE && commas+1 < Self::SUFFIXES.len() {
            rem /= Self::SIZE;
            commas += 1;
        }
        /* let (rem, commas) = Self::SUFFIXES.fold((self.0,0), |rem,commas| {
            if rem < Self::SIZE { (rem, commas) } else { todo!() }
            // if rem < Self::SIZE && commas < Self::SUFFIXES.len()
        }); */
        write!(f, "{}{}{}", rem, Self::DELIM, Self::SUFFIXES[commas])
    }
}
#[cfg(test)]
mod tests {
    use super::ByteSize;
    #[test]
    fn byte_size() {
        let bs = |x| ByteSize(x).to_string();
        assert_eq!("0b", bs(0));
        assert_eq!("123b", bs(123));
        assert_eq!("123kb", bs(123 * 1024));
        assert_eq!("123mb", bs((123 * 1024) * 1024));
        assert_eq!("123000mb", bs(((123 * 1000) * 1024) * 1024));
    }
}




use mockall::*;
use mockall::predicate::*;

#[automock]
pub trait Callable {
    // type Result = Result<T, E>;
    // fn call(&mut self) -> Self::Result;
}

#[derive(Debug)]
struct MockFn {
    calls: usize, 
}



