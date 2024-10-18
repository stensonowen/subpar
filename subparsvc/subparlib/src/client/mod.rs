
use {hyper::{self, body::Bytes}, hyper_tls};
use anyhow::{Result, Context as _};
use super::{gtfs};
use tokio::time::{Duration, timeout};

mod feed;
pub use feed::Feed;

mod listener;
pub use listener::{Listener, Response};


const TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug)]
pub struct Client {
    client: hyper::Client<Https>,
    timeout: Duration,
    api_key: String,
}

impl Default for Client {
    fn default() -> Client {
        let key = "please".split_whitespace().next().unwrap();
        Client::new(key.into())
    }
}

type Request = hyper::Request<hyper::Body>;
type Https = hyper_tls::HttpsConnector<hyper::client::HttpConnector>;

impl Client {

    pub fn new(api_key: String) -> Self {
        let client = hyper::Client::builder().build(Https::new());
        Client { client, api_key, timeout: TIMEOUT }
    }

    fn make_req(&self, feed: &hyper::Uri) -> Result<Request> {
        hyper::Request::builder()
            .header("x-api-key", &self.api_key)
            .uri(feed)
            .body(hyper::Body::default())
            .map_err(anyhow::Error::from)
            .context("request build")
    }

    pub async fn test(&self) -> Result<Bytes> {
        let url = "https://icanhazip.com";
        let cli = hyper::Client::builder().build::<_, hyper::Body>(
            hyper_tls::HttpsConnector::new());
        let ret = cli.get(url.parse()?).await?;
        hyper::body::to_bytes(ret.into_body()).await.context("extract")
    }

    pub async fn fetch(&self, url: &hyper::Uri) -> Result<Bytes> {
        let req = self.make_req(&*url).expect("bad feed");
        let resp = self.client.request(req).await.context("req")?;
        tracing::debug!("status {}", resp.status());
        let body = resp.into_body();
        let data = hyper::body::to_bytes(body).await.context("extract")?;
        Ok(data)
    }

    pub async fn fetch2(&self, url: &hyper::Uri) -> Result<Bytes> {
        let req = self.make_req(&url).expect("bad feed");
        let fut = self.client.request(req);
        let resp = match timeout(self.timeout, fut).await.context("request") {
            Ok(Ok(r)) => r,
            Ok(Err(e)) => return Err(e.into()),
            Err(_) => anyhow::bail!("Request timed out 1 {:?}", self.timeout),
        };
        tracing::debug!("status {}", resp.status());
        let body = resp.into_body();
        let fut = hyper::body::to_bytes(body);
        match timeout(self.timeout, fut).await.context("extract") {
            Ok(Ok(x)) => Ok(x),
            Ok(Err(e)) => Err(e.into()),
            Err(_) => anyhow::bail!("Request to_bytes timeout 2"),
        }
    }

    pub async fn get(&self, url: &'static str) -> Result<gtfs::FeedMessage> {
        let feed = hyper::Uri::from_static(url);
        let data = self.fetch(&feed).await?;
        Ok(protobuf::Message::parse_from_bytes(&data)?)
    }

}

