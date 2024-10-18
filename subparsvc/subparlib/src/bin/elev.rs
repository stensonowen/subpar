use subpar::{
    ApiClient,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    tracing_subscriber::fmt::init();
    let client = ApiClient::default();
    let data = client.get_outage().await?;
    for (i, msg) in data.iter().enumerate() {
        println!("{i:<3}  {:?}", msg);
    }
    Ok(())
}


