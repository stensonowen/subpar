
use anyhow::{Context};
use super::{ResponseUuid};
use subpar::msg::Position;


#[derive(Debug, PartialEq, Clone)]
pub struct PositionId(i32);


impl super::Table<Position> {
    pub async fn create(&self) -> anyhow::Result<()> {
        let sql = format!(r#"
CREATE TABLE {}
( msg_id    SERIAL  NOT NULL    PRIMARY KEY
, response  UUID REFERENCES responses(uuid)
, modified  TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
, trip      VARCHAR(24)
, date      DATE NOT NULL
, time      TIMESTAMP WITH TIME ZONE NOT NULL
, stop      VARCHAR(4) NOT NULL
, stop_n    INT
, status    INT
);"#, self.name);
        self.execute(&sql).await.context(sql)?;
        Ok(())
    }

    pub async fn insert(&self, response: ResponseUuid, position: &Position) -> anyhow::Result<PositionId> {
        let sql = format!(r#"
INSERT INTO {}
( trip
, date
, time
, stop
, stop_n
, status
, response
) VALUES ( $1, $2, $3, $4, $5, $6, $7
) RETURNING msg_id;"#, self.name);
       let result = self.client()
           .await?
           .query_one(&sql, &[
                &position.trip.as_str(),
                &position.trip.date(),
                &position.time.as_utc(),
                &position.stop.as_ref(),
                &position.stop_n.map(|x| x as i32),
                &(position.status as i32),
                &*response,
           ]).await.context(sql)?;
       Ok(PositionId(result.get(0)))
    }
}


/*
impl TryFrom<&pg::Row> for PositionRow {
    type Error = anyhow::Error;
    fn try_from(row: &pg::Row) -> anyhow::Result<PositionRow> {
        let id = row.try_get("msg_id").map(PositionId)
            .map_err(|e| anyhow!("pos row missing msg_id: {e}"))?;
        let modified = row.try_get("modified")
            .map(Timestamp::from_utc)
            .map_err(|e| anyhow!("pos row missing modified: {e}"))?;
        let date = row.try_get("date")
            .map(Date::new)
            .map_err(|e| anyhow!("pos row missing date: {e}"))?;
        let trip = row.try_get("trip")
            .map_err(|e| anyhow!("pos row trip: {e}"))
            .and_then(|s| TripId::parse(s, date))?;
        let time = row.try_get("time")
            .map(Timestamp::from_utc)
            .map_err(|e| anyhow!("pos row time: {e}"))?;
        let stop: &str = row.try_get("stop")
            .map_err(|e| anyhow!("pos row stop: {e}"))?;
        let stop = stop.parse()?;
        let stop_n: Option<i32> = row.try_get("stop_n")
            .map_err(|e| anyhow!("pos row stop_n: {e}"))?;
        let stop_n = stop_n.map(|x| x as u32);
        let status = row.try_get("status")
            .map_err(|e| anyhow!("pos row status: {e}"))
            .and_then(|x: i32| PositionStatus::try_from(x as u32))?;
        Ok(PositionRow {
            id,
            modified,
            pos: Position { trip, time, stop, status, stop_n, }
        })
    }
}
*/
