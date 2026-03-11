use crate::{error::ServiceResult, repository::event::services::EventReducerContext};
use spacetimedb::ReducerContext;

pub mod character;
pub mod chat;
pub mod event;
pub mod user;
pub mod world;

pub fn init(ctx: &ReducerContext) {
    ctx.publish().system_init();
}

pub fn identity_connected(ctx: &ReducerContext) -> ServiceResult<()> {
    ctx.publish().user_signed_in(ctx.sender())?;
    Ok(())
}

pub fn identity_disconnected(ctx: &ReducerContext) {
    ctx.publish().user_signed_out(ctx.sender());
}
