use crate::{
    error::ServiceResult,
    extend::validate::ReducerContextRequirements,
    repository::world::{services::WorldReducerContext, types::MovementV1},
};
use spacetimedb::{ReducerContext, reducer};

#[reducer]
pub fn move_character_v1(ctx: &ReducerContext, movement: MovementV1) -> ServiceResult<()> {
    let character = ctx.require_online()?;
    ctx.world_services().move_character(character.character_id, movement)?;
    Ok(())
}
