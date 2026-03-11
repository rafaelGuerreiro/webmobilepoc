use crate::{error::ServiceResult, repository::event::services::EventReducerContext};
use item::services::ItemReducerContext;
use spacetimedb::ReducerContext;

pub mod character;
pub mod chat;
pub mod event;
pub mod user;
pub mod world;

pub fn init(ctx: &ReducerContext) {
    ctx.publish().system_init();
    ctx.item_services().seed_item_definitions();
    ctx.item_services().seed_test_floor_items();
}

pub fn identity_connected(ctx: &ReducerContext) -> ServiceResult<()> {
    ctx.publish().user_signed_in(ctx.sender())?;
    Ok(())
}

pub fn identity_disconnected(ctx: &ReducerContext) {
    ctx.publish().user_signed_out(ctx.sender());
}
