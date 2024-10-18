
use anyhow::Context;
use subpar::{Timestamp, msg::{Schedule, StopPlan}};
use super::ResponseUuid;


#[derive(Debug, PartialEq, Clone)]
pub struct ScheduleId(i32);

impl super::Table<Schedule> {
    const TABLE_NAME: &'static str = "schedules";
    pub async fn create(&self) -> anyhow::Result<()> {
        let result = self.execute(&format!(r#"
CREATE TABLE {}
( sched_id  SERIAL NOT NULL PRIMARY KEY
, response  UUID REFERENCES responses(uuid)
, modified  TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
, trip      VARCHAR(20) NOT NULL
, origin    TIMESTAMP WITH TIME ZONE NOT NULL
); "#, Self::TABLE_NAME))
            .await.context("failed to create schedules table")?;
        Ok(())
    }
    pub async fn insert(&self, response: ResponseUuid, schedule: &Schedule) -> anyhow::Result<ScheduleId> {
        let sql = format!(r#"
INSERT INTO {}
( response, trip, origin )
VALUES ( $1, $2, $3 )
RETURNING sched_id;"#, self.name);
        let result = self.client()
            .await?
            .query_one(&sql, &[
                &*response,
                &schedule.trip().as_str(),
                &schedule.trip().origin().as_utc(),
            ]).await
            .context(sql)?;
        Ok(ScheduleId(result.get(0)))
    }

}


#[derive(Debug, PartialEq, Clone)]
pub struct StopPlanId(i32);

impl super::Table<StopPlan> {
    pub async fn create(&self) -> anyhow::Result<()> {
        let result = self.execute(&format!(r#"
CREATE TABLE {}
( plan_id   SERIAL NOT NULL PRIMARY KEY
, schedule  INT REFERENCES schedules(sched_id)
, modified  TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
, stop      VARCHAR(4) NOT NULL
, arrive    TIMESTAMP WITH TIME ZONE
, depart    TIMESTAMP WITH TIME ZONE
);"#, self.name)).await.context("stopplans.create")?;
        Ok(())
    }
    pub async fn insert_all(&self, id: &ScheduleId, schedule: &Schedule) -> anyhow::Result<()> {
        let mut client = self.client().await?;
        let tr = client.transaction().await?;
        for stop in schedule.stops() {
            let sql = format!(r#"
INSERT INTO {}
( schedule
, stop
, arrive
, depart
) VALUES ( $1, $2, $3, $4
);"#, self.name);
            tr.execute(&sql, &[
                &id.0,
                &stop.id.as_ref(),
                &stop.times.arr().map(Timestamp::as_utc),
                &stop.times.dep().map(Timestamp::as_utc),
            ]).await.context("stopplans.insert")?;
        }
        Ok(tr.commit().await?)
    }
}

