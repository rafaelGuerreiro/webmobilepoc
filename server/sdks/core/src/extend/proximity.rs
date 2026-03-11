use crate::{
    constants::{MAP_VIEW_RADIUS, SECTOR_SIZE},
    repository::{
        character::online_character_v1__view,
        world::{
            OccupiedTileV1, occupied_tile_v1__view, online_character_position_v1__view,
            types::{Rect, Vec2},
        },
    },
};
use spacetimedb::ViewContext;

pub fn find_range(ctx: &ViewContext) -> Option<Rect> {
    let current = ctx.db.online_character_v1().user_id().find(ctx.sender())?;
    let position = ctx
        .db
        .online_character_position_v1()
        .character_id()
        .find(current.character_id)?;

    Some(Rect::new(
        position.x.saturating_sub(MAP_VIEW_RADIUS),
        position.y.saturating_sub(MAP_VIEW_RADIUS),
        position.x.saturating_add(MAP_VIEW_RADIUS),
        position.y.saturating_add(MAP_VIEW_RADIUS),
    ))
}

pub fn iter_nearby_occupied(ctx: &ViewContext) -> Vec<OccupiedTileV1> {
    let Some(rect) = find_range(ctx) else {
        return Vec::new();
    };

    let sec_min_x = rect.min.x / SECTOR_SIZE;
    let sec_max_x = rect.max.x / SECTOR_SIZE;
    let sec_min_y = rect.min.y / SECTOR_SIZE;
    let sec_max_y = rect.max.y / SECTOR_SIZE;

    let mut occupied = Vec::new();
    for sx in sec_min_x..=sec_max_x {
        for sy in sec_min_y..=sec_max_y {
            let sector_key = ((sx as u64) << 16) | (sy as u64);
            for tile in ctx.db.occupied_tile_v1().sector_key().filter(sector_key) {
                let pos = Vec2::from_tile_id(tile.tile_id);
                if rect.contains(pos) {
                    occupied.push(tile);
                }
            }
        }
    }
    occupied
}
