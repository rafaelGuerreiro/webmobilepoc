use crate::{
    constants::SECTOR_SIZE,
    extend::proximity::{find_ranges, iter_nearby_occupied},
    repository::{
        character::{CharacterV1, character_v1__view, online_character_v1__view},
        world::{CharacterPositionV1, MapV1, map_v1__view, online_character_position_v1__view, types::Rect},
    },
};
use spacetimedb::{ViewContext, view};

#[view(accessor = vw_world_map_v1, public)]
pub fn vw_world_map_v1(ctx: &ViewContext) -> Vec<MapV1> {
    let Some((rect, (min_z, max_z))) = find_ranges(ctx) else {
        return Vec::new();
    };

    let sec_min_x = rect.min.x / SECTOR_SIZE;
    let sec_max_x = rect.max.x / SECTOR_SIZE;
    let sec_min_y = rect.min.y / SECTOR_SIZE;
    let sec_max_y = rect.max.y / SECTOR_SIZE;

    (min_z..=max_z)
        .flat_map(|z| {
            (sec_min_x..=sec_max_x).flat_map(move |sx| {
                (sec_min_y..=sec_max_y).map(move |sy| ((z as u64) << 32) | ((sx as u64) << 16) | (sy as u64))
            })
        })
        .flat_map(|sector_key| ctx.db.map_v1().sector_key().filter(sector_key))
        .filter(|chunk| chunk.z >= min_z && chunk.z <= max_z && Rect::from(chunk).overlaps(&rect))
        .collect()
}

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
