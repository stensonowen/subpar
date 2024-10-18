
/*
 * todo
 * rotatiing/zip logs
 * gzipping responses
 * s/println/log
 * pass port in argv?
 * warnings
 *
 */

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    
    subparweb::serve().await?;
    Ok(())
}

/*
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = axum::Router::new().route("/", axum::routing::get(handler));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    Ok(())
}
async fn handler() -> &'static str {
    "axum says hell world"
}
*/
