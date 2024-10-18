use axum::{ Router, routing::{get}, extract::{Json, Path, State}, body::Body, http::StatusCode, };
use std::{time::Duration};
use subpar::{api::ComplexId, ApiClient, Listener, Feed, state::{States, Elevator, Upcoming, ComplexFull, ElevatorSummary }};
use tokio_stream::StreamExt as _;
use tokio::{fs, net::TcpListener};
use tracing::{info, debug, warn, error};
use tower_http::cors;
use http::{Method, header::HeaderValue};

const FEED_POLL_PERIOD: Duration = Duration::new(10, 0);

pub async fn serve() -> anyhow::Result<()> {
    let client = ApiClient::default();
    let state = {
        let complexes = client.get_complexes().await?;
        let elevators = client.get_equipment().await?;
        let outages = client.get_outages_nocache().await?;
        let entrances = client.get_entrances().await?;
        States::new(&complexes, &elevators, &outages, &entrances)
    };
    let cors = cors::CorsLayer::new()
        .allow_methods([Method::GET])
        .allow_origin(HeaderValue::from_static("https://api.subpar.nyc"));
    let app = Router::new()
        .route("/upcoming/:id", get(get_trains))
        .route("/elevators/:id", get(get_elevators))
        .route("/elevators_overview", get(get_elevators_overview))
        .route("/complex/:id", get(get_complex_api))
        .route("/c/:id", get(get_complex_page))
        .layer(cors)
        .with_state(state.clone());
    tokio::spawn(populate_feeds(state.clone()));
    tokio::spawn(poll_elevators(client, state.clone()));
    webserver("0.0.0.0:3000", app).await;
    Ok(())
}

async fn webserver(addr: &str, app: Router) {
    info!("listening at {addr}");
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
        debug!(%rsp.feed, "feed update");
        state.trains.update(&rsp);
    }
}

async fn poll_elevators(client: ApiClient, state: States) {
    let mut interval = tokio::time::interval(Duration::from_secs(60 * 60));
    loop {
        match client.get_outages_nocache().await {
            Ok(o) => {
                info!("Updated 'outages'");
                state.elevators.update(o.as_ref())
            },
            Err(e) => error!("Outage poll error: {e}"),
        }
        interval.tick().await;
    }
}

async fn get_elevators(
    Path(id): Path<ComplexId>,
    State(state): State<States>,
) -> Result<Json< Vec<Elevator> >, (StatusCode, String)> {
    match state.elevators.get(id) {
        Some(e) => {
            info!(%id, "serving elevator list");
            Ok(Json(e))
        },
        None => {
            warn!(%id, "no such elevators");
            Err((StatusCode::NOT_FOUND, format!("complex '{id}' not found")))
        }
    }
}

async fn get_elevators_overview(
    State(state): State<States>,
) -> Json<ElevatorSummary> {
    Json(state.elevators.get_summary())
}

async fn get_trains(
    Path(id): Path<ComplexId>,
    State(state): State<States>,
) -> Result<Json< Vec<Upcoming> >, (StatusCode, String)> {
    match state.trains.get(id) {
        Some(u) => {
            info!(%id, "serving upcoming");
            Ok(Json(u))
        },
        None => {
            warn!(%id, "no such upcoming");
            Err((StatusCode::NOT_FOUND, format!("complex '{id}' not found")))
        }
    }
}

async fn get_complex_page() -> Result<Body, (StatusCode, String)> {
    fs::read_to_string("ui/index.html").await
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

