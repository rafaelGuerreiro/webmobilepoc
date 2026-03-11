use crate::{
    error::ServiceResult,
    repository::character::{
        services::CharacterReducerContext,
        types::{GenderV1, RaceV1},
    },
};
use spacetimedb::{ReducerContext, reducer};

#[reducer]
pub fn create_character_v1(ctx: &ReducerContext, display_name: String, gender: GenderV1, race: RaceV1) -> ServiceResult<()> {
    ctx.character_services()
        .create_character(ctx.sender(), display_name, gender, race)?;
    Ok(())
}

#[reducer]
pub fn select_character_v1(ctx: &ReducerContext, character_id: u64) -> ServiceResult<()> {
    ctx.character_services().select_character(ctx.sender(), character_id)?;
    Ok(())
}

#[reducer]
pub fn unselect_character_v1(ctx: &ReducerContext) -> ServiceResult<()> {
    ctx.character_services().unselect_character(ctx.sender())?;
    Ok(())
}
