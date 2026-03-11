use crate::{
    extend::proximity::iter_nearby_occupied,
    repository::{
        character::{CharacterV1, character_v1__view, online_character_v1__view},
        world::{CharacterPositionV1, online_character_position_v1__view},
    },
};
use spacetimedb::{ViewContext, view};

#[view(accessor = vw_world_my_character_position_v1, public)]
pub fn vw_world_my_character_position_v1(ctx: &ViewContext) -> Option<CharacterPositionV1> {
    let current = ctx.db.online_character_v1().user_id().find(ctx.sender())?;
    ctx.db
        .online_character_position_v1()
        .character_id()
        .find(current.character_id)
}

#[view(accessor = vw_nearby_characters_v1, public)]
pub fn vw_nearby_characters_v1(ctx: &ViewContext) -> Vec<CharacterV1> {
    iter_nearby_occupied(ctx)
        .into_iter()
        .flat_map(|occupied| occupied.character_ids.into_iter())
        .filter(|character_id| {
            ctx.db
                .online_character_position_v1()
                .character_id()
                .find(*character_id)
                .is_some()
        })
        .filter_map(|character_id| ctx.db.character_v1().character_id().find(character_id))
        .collect()
}

#[view(accessor = vw_nearby_character_positions_v1, public)]
pub fn vw_nearby_character_positions_v1(ctx: &ViewContext) -> Vec<CharacterPositionV1> {
    iter_nearby_occupied(ctx)
        .into_iter()
        .flat_map(|occupied| occupied.character_ids.into_iter())
        .filter_map(|character_id| ctx.db.online_character_position_v1().character_id().find(character_id))
        .collect()
}
