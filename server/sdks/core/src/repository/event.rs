use crate::{
    error::ServiceResult,
    extend::validate::ReducerContextRequirements,
    repository::event::{services::EventReducerContext, types::DeferredEventV1},
};
use spacetimedb::{Identity, ReducerContext, ScheduleAt, Timestamp, reducer, table};

pub mod services;
pub mod types;

#[table(accessor = oneshot_deferred_event_v1, private, scheduled(oneshot_deferred_event_scheduled_v1))]
#[derive(Debug)]
pub struct OneshotDeferredEventV1 {
    #[auto_inc]
    #[primary_key]
    pub job_id: u64,
    pub scheduled_at: ScheduleAt,
    pub event: DeferredEventV1,
    pub sender: Identity,
    pub created_at: Timestamp,
}

#[reducer]
pub fn oneshot_deferred_event_scheduled_v1(ctx: &ReducerContext, timer: OneshotDeferredEventV1) -> ServiceResult<()> {
    ctx.require_internal_access()?;
    ctx.event_services().handle_deferred_event(timer);
    Ok(())
}
