use axum::{ Router, routing::{get}, extract::{Json, Path, Query, State}, body::Body, http::StatusCode, };
use std::{time::Duration};
use crate::{api::ComplexId, ApiClient, Listener, Feed, state::{States, Elevator, Upcoming, ComplexFull, }};
use tokio_stream::StreamExt as _;
use tokio::{fs, net::TcpListener};

const FEED_POLL_PERIOD: Duration = Duration::new(10, 0);

pub async fn serve() -> anyhow::Result<()> {
    let client = ApiClient::default();
    let state = {
        let complexes = client.get_complexes().await?;
        let elevators = client.get_equipment().await?;
        let outages = client.get_outage().await?;
        let entrances = client.get_entrances().await?;
        States::new(&complexes, &elevators, &outages, &entrances)
    };
    let app = Router::new()
        .route("/hello", get(hello))
        .route("/f/:name", get(get_file))
        .route("/upcoming/:id", get(get_trains))
        .route("/elevators/:id", get(get_elevators))
        .route("/complex/:id", get(get_complex_api))
        .route("/c/:id", get(get_complex_page))
        .route("/favicon.ico", get(get_favicon))
        .with_state(state.clone());
    tokio::spawn(populate_feeds(state.clone()));
    tokio::spawn(poll_elevators(client, state.clone()));
    webserver("0.0.0.0:3000", app).await;
    Ok(())
}

async fn webserver(addr: &str, app: Router) {
    println!("listening at {addr}");
    let listener = TcpListener::bind(addr).await.unwrap();
    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("axum error {e}");
    }
}

async fn populate_feeds(state: States) {
    let feeds = "l 1234567 ace bdfm g jz nqrw si".split(" ")
        .map(Feed::from_static)
        .collect();
    let mut listener = Listener::new(Default::default(), feeds, FEED_POLL_PERIOD).spawn();
    while let Some(rsp) = listener.next().await {
        state.trains.update(&rsp);
    }
}

async fn poll_elevators(client: ApiClient, state: States) {
    let mut interval = tokio::time::interval(Duration::from_secs(60 * 60));
    loop {
        // todo need a get() version that doesn't just read the file
        match client.get_outages_nocache().await {
            Ok(o) => {
                println!("Fetched 'outages'");
                state.elevators.update(o.as_ref())
            },
            Err(e) => println!("elevator poll error: {e}"),
        }
        interval.tick().await;
    }
}

#[derive(serde::Deserialize)]
struct ComplexId {
    complex: ComplexId,
}

async fn get_elevators(
    Path(id): Path<ComplexId>,
    State(state): State<States>,
) -> Result<Json< Vec<Elevator> >, (StatusCode, String)> {
    match state.elevators.get(id) {
        Some(e) => Ok(Json(e)),
        None => Err((StatusCode::NOT_FOUND, format!("complex '{id}' not found"))),
    }
}

async fn get_trains(
    Path(id): Path<ComplexId>,
    State(state): State<States>,
) -> Result<Json< Vec<Upcoming> >, (StatusCode, String)> {
    match state.trains.get(id) {
        Some(u) => Ok(Json(u)),
        None => Err((StatusCode::NOT_FOUND, format!("complex '{id}' not found"))),
    }
}

async fn hello() -> &'static str {
    "hell world"
}

async fn get_complex_page() -> Result<Body, (StatusCode, String)> {
    fs::read_to_string("ui/index.html").await
        .map(Body::new)
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))
}

async fn get_favicon() -> Result<Body, (StatusCode, String)> {
    fs::read_to_string("ui/elevator4.svg").await
        .map(Body::new)
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))
}

async fn get_complex_api(
    Path(id): Path<ComplexId>,
    State(state): State<States>,
) -> Result<Json< ComplexFull >, (StatusCode, String)> {
    match state.get_full(id) {
        Some(x) => Ok(Json(x)),
        None => Err((StatusCode::NOT_FOUND, format!("complex '{id}' not found"))),
    }
}

async fn get_file(Path(name): Path<String>) -> Result<Body, (StatusCode, String)> {
    tracing::debug!("get file {name}");
    assert!(!name.starts_with(".."));
    let path = format!("ui/{name}");
    match fs::read_to_string(&path).await {
        Ok(s) => Ok(Body::new(s)),
        Err(e) => Err((StatusCode::NOT_FOUND, e.to_string())),
    }
}

