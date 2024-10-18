
use tokio_postgres as pg;
use std::marker::PhantomData;
use deadpool_postgres::{Pool, Object as PgObject};
use tracing::{error, info};
use subpar::{Response, msg::{Schedule, StopPlan, Position}};
use anyhow::Context;

mod position;
pub use position::PositionId;

mod schedule;
pub use schedule::ScheduleId;

mod response;
pub use response::ResponseUuid;

pub async fn connect() -> Result<pg::Client, pg::Error> {
    let s = "host=localhost user=postgres password=sergtsop";
    let (client, connection) = pg::connect(s, pg::NoTls).await?;
    tokio::spawn(async {
        info!("db connection open");
        if let Err(e) = connection.await {
            error!("db connection error: {e}");
        }
        info!("db connection closed");
    });
    Ok(client)
}

pub struct Db {
    pub responses: Table<Response>,
    pub schedules: Table<Schedule>,
    pub stopplans: Table<StopPlan>,
    pub positions: Table<Position>,
}

impl From<Pool> for Db {
    fn from(pool: Pool) -> Self {
        Db {
            responses: Table::new(pool.clone(), "responses"),
            schedules: Table::new(pool.clone(), "schedules"),
            stopplans: Table::new(pool.clone(), "stopplans"),
            positions: Table::new(pool.clone(), "positions"),
        }
    }
}
impl Db {
    pub fn new(user: String, pass: String, host: String) -> anyhow::Result<Self> {
        let pool = deadpool_postgres::Config {
            user: Some(user),
            password: Some(pass),
            host: Some(host),
            dbname: Some("postgres".to_string()),
            ..Default::default()
        }.create_pool(None, pg::NoTls)?;
        Ok(Db::from(pool))
    }
    pub async fn reset_all(&self) -> anyhow::Result<()> {
        // order matters because of foreign keys
        eprintln!("DROPPING TABLES");
        self.stopplans.try_drop().await?;
        self.schedules.try_drop().await?;
        self.positions.try_drop().await?;
        self.responses.try_drop().await?;
        self.responses.create().await?;
        self.positions.create().await?;
        self.schedules.create().await?;
        self.stopplans.create().await?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct Table<T> {
    pool: Pool,
    name: &'static str,
    phantom: PhantomData<T>,
}

impl<T> Table<T> {
    fn new(pool: Pool, name: &'static str) -> Self {
        Table { pool, name, phantom: PhantomData }
    }
    async fn execute(&self, query: &str) -> anyhow::Result<()> {
        let client = self.pool.get().await.context("pool.get")?;
        client.execute(query, &[]).await.context("client.execute").map(|_| ())
    }
    async fn _query1(&self, query: &str) -> anyhow::Result<pg::Row> {
        let client = self.client().await?;
        client.query_one(query, &[]).await.context("client.query")
    }
    async fn client(&self) -> anyhow::Result<PgObject> {
        Ok(self.pool.get().await.context("pool.get")?)
    }
    pub async fn try_drop(&self) -> anyhow::Result<()> {
        self.execute(&format!("DROP TABLE IF EXISTS {};", self.name)).await
    }
}
