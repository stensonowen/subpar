use super::gtfs_realtime;
use anyhow::{anyhow, Context as _};
use crate::{msg, Timestamp};

pub trait FromGtfs: Sized {
    type In;
    fn parse(g: &Self::In) -> anyhow::Result<Self>;
    // fn check(x: Self::In) -> anyhow::Result< () > { Ok( () ) }
}

use crate::msg::{StopPlan, PositionStatus, Position, Schedule, Update};

#[macro_export]
macro_rules! pbget {

    ( $( $cond:expr $(,)? )* => $val:expr, $( $log:expr $(,)? )* ) => {{
        let mut succ = true;
        $( if ( $cond == false ) { succ = false; })*
        if !succ {
            return Err(::anyhow::anyhow!(
                "Failed to get '{}' because check failed", stringify!( $val )
                    ).context(format!( $( $log )+)));
        }
        $val
    }};
    ( $( $cond:expr $(,)? )* => $val:expr ) => {{
        $( ::anyhow::ensure!( $cond,
                "Failed to get '{}' because check failed: '{}'",
                stringify!( $val ), stringify!( $cond ));
        )* $val
    }};

}

impl FromGtfs for msg::Batch {
    type In = gtfs_realtime::FeedMessage;
    fn parse(g: &Self::In) -> anyhow::Result<Self> {
        let head = pbget!( g.has_header() => g.get_header(), "{g:?}" );
        let time = pbget!( head.has_timestamp() => head.get_timestamp() );
        let time = Timestamp::from_unix(time.try_into().unwrap());
        let msgs = g.get_entity().iter().map(Update::parse).collect();
        Ok(msg::Batch { time, msgs })
    }
}

impl FromGtfs for Update {
    type In = gtfs_realtime::FeedEntity;
    fn parse(g: &Self::In) -> anyhow::Result<Self> {
        const T: bool = true;
        const F: bool = false;
        match [g.has_trip_update(), g.has_vehicle(), g.has_alert()] {
            [T, F, F] => Ok(Update::Schedule(Schedule::parse(g.get_trip_update())?)),
            [F, T, F] => Ok(Update::Position(Position::parse(g.get_vehicle())?)),
            [F, F, T] => Ok(Update::Alert),
            [F, F, F] => Err(anyhow!("FeedEntity unrecognized")),
            [t, v, a] => Err(anyhow!("FeedEntity multiple: trip={t} pos={v} alrt={a}")),
        }
        .with_context(|| format!("feed entity {g:?}"))
    }
}

// impl FromGtfs for update::StopPlan {
//     type In = gtfs_realtime::TripUpdate_StopTimeUpdate;
//     fn parse(g: &Self::In) -> anyhow::Result<Self> {
//         let kind = pbget!( g.has_schedule_relationship() => g.get_schedule_relationship() );
//         let id = pbget!( g.has_stop_id() => g.get_stop_id() )
//             .parse().with_context(|| format!("stop time update {g:?}"))?;
//         let arr = opt(g.has_arrival(), g.get_arrival(), make_time)?;
//         let dep = opt(g.has_departure(), g.get_departure(), make_time)?;
//         let times = update::Times::new(todo!(), todo!())?;
//         Ok(StopPlan::new(id, times))
//     }
// }

impl FromGtfs for Position {
    type In = gtfs_realtime::VehiclePosition;
    fn parse(g: &Self::In) -> anyhow::Result<Self> {
        let trip = pbget!( g.has_trip() => g.get_trip() );
        let id = pbget!( trip.has_trip_id() => trip.get_trip_id() );
        let start = {
            let s = pbget!( trip.has_start_date() => trip.get_start_date() );
            let t = Timestamp::from_yyyymmdd(s)?;
            t.date()
        };
        let trip = msg::TripId::parse(id, start)?;
        // let stop_n = common::StopN::from(pbget!( g.has_current_stop_sequence() => g.get_current_stop_sequence()));
        let stop_n = match (g.has_current_stop_sequence(), g.get_current_stop_sequence()) {
            (false, _) => None,
            (true, n) => Some(n),
        };
        let stop = pbget!( g.has_stop_id() => g.get_stop_id()).parse()?;
        let time = Timestamp::from_unix(pbget!( g.has_timestamp() => g.get_timestamp().try_into().unwrap() ));
        use gtfs_realtime::VehiclePosition_VehicleStopStatus as SS;
        let status = match (g.has_current_status(), g.get_current_status()) {
            (false, _) => PositionStatus::Nothing,
            (true, SS::STOPPED_AT) => PositionStatus::At,
            (true, SS::INCOMING_AT) => PositionStatus::Near,
            (true, SS::IN_TRANSIT_TO) => PositionStatus::EnRoute,
        };
        Ok(Position::new(trip, stop, stop_n, status, time))
    }
}

fn opt<T, F, R>(cond: bool, val: T, func: F) -> anyhow::Result<Option<R>>
where
    F: FnOnce(T) -> anyhow::Result<R>,
{
    if cond {
        Ok(Some(func(val)?))
    } else {
        Ok(None)
    }
}
fn make_time(g: &gtfs_realtime::TripUpdate_StopTimeEvent) -> anyhow::Result<Timestamp> {
    let unix = pbget!( g.has_time() => g.get_time() );
    Ok(Timestamp::from_unix(unix))
}

impl FromGtfs for Schedule {
    type In = gtfs_realtime::TripUpdate;
    fn parse(g: &Self::In) -> anyhow::Result<Self> {
        let trip = pbget!( g.has_trip() => g.get_trip() );
        // assert!(g.has_timestamp() == false);
        let time = Timestamp::epoch();
        let id = pbget!( trip.has_trip_id() => trip.get_trip_id() );
        let start = {
            let s = pbget!( trip.has_start_date() => trip.get_start_date() );
            let t = Timestamp::from_yyyymmdd(s)?;
            t.date()
        };
        let trip_id = msg::TripId::parse(id, start)?;
        let upds = g
            .get_stop_time_update()
            .iter()
            .map(|x| {
                let id = pbget!( x.has_stop_id() => x.get_stop_id() ).parse()?;
                // if x.has_stop_sequence() { log::info!("Found a stop update w/ stop_n: {x:?}") }
                let arr = opt(x.has_arrival(), x.get_arrival(), make_time)?;
                let dep = opt(x.has_departure(), x.get_departure(), make_time)?;
                let times = msg::Times::new(arr, dep)?;
                Ok(StopPlan::new(id, times))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Schedule::new(trip_id, time, upds))
    }
}

//     // message fields
//     stop_sequence: ::std::option::Option<u32>,
//     stop_id: ::protobuf::SingularField<::std::string::String>,
//     pub arrival: ::protobuf::SingularPtrField<TripUpdate_StopTimeEvent>,
//     pub departure: ::protobuf::SingularPtrField<TripUpdate_StopTimeEvent>,
//     schedule_relationship: ::std::option::Option<TripUpdate_StopTimeUpdate_ScheduleRelationship>,
//     // special fields
//     pub unknown_fields: ::protobuf::UnknownFields,
//     pub cached_size: ::protobuf::CachedSize,
// }

#[cfg(test)]
mod tests {
    
    #[test]
    fn getter_a() -> anyhow::Result<()> {
        let a = pbget!( 42 > 10, true != false, 4 == 4, true
            => 1 + 2 + 3);
        assert_eq!(a, 6);

        Ok(())
    }
}
