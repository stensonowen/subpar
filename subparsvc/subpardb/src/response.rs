
use anyhow::{Context};
use subpar::{Response};
use uuid::Uuid;
use std::{ops, fmt};


#[derive(Debug, Clone, Copy)]
pub struct ResponseUuid(Uuid);

impl super::Table<Response> {
    pub async fn create(&self) -> anyhow::Result<()> {
        self.execute(&format!(r#"
CREATE TABLE {}
( uuid      UUID PRIMARY KEY
, modified  TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
, feed      VARCHAR(8) NOT NULL
, request   TIMESTAMP WITH TIME ZONE
, response  TIMESTAMP WITH TIME ZONE
, count     INT DEFAULT 1
);"#, self.name))
            .await.context("failed create table responses;")?;
        Ok(())
    }
    /// Returns Ok(Some(Uuid)) if a new row was inserted.
    pub async fn upsert(&self, x: &Response) -> anyhow::Result<Option<ResponseUuid>> {
        let sql = format!(r#"
INSERT INTO {table} 
( uuid, feed, request, response )
VALUES ( $1, $2, $3, $4 )
ON CONFLICT (uuid) DO UPDATE
SET count = {table}.count + EXCLUDED.count
, modified = NOW()
RETURNING uuid, count;"#, table = self.name);
        let client = self.client().await?;
        let result = client.query_one(&sql, &[
                &x.hash,
                &x.feed.name(),
                &x.t_req.as_utc(),
                &x.t_rsp.as_utc()
            ]).await.context("response.upsert")?;
        let count: i32 = result.get("count");
        if count == 1 {
            Ok(Some(ResponseUuid(result.get("uuid"))))
        } else {
            Ok(None)
        }
    }
}

impl fmt::Display for ResponseUuid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Response[{}]", self.0)
    }
}

impl ops::Deref for ResponseUuid {
    type Target = Uuid;
    fn deref(&self) -> &Uuid { &self.0 }
}
