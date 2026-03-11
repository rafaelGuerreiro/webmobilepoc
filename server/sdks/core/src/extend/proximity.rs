use crate::{
    constants::{MAP_VIEW_RADIUS, SECTOR_SIZE},
    repository::{
        character::online_character_v1__view,
        world::{
            OccupiedTileV1, occupied_tile_v1__view, online_character_position_v1__view,
            types::{Rect, Vec3},
        },
    },
};
use spacetimedb::ViewContext;

pub type ZRange = (u8, u8);

pub fn find_ranges(ctx: &ViewContext) -> Option<(Rect, ZRange)> {
    let current = ctx.db.online_character_v1().user_id().find(ctx.sender())?;
    let position = ctx
        .db
        .online_character_position_v1()
        .character_id()
        .find(current.character_id)?;

    let rect = Rect::new(
        position.x.saturating_sub(MAP_VIEW_RADIUS),
        position.y.saturating_sub(MAP_VIEW_RADIUS),
        position.x.saturating_add(MAP_VIEW_RADIUS),
        position.y.saturating_add(MAP_VIEW_RADIUS),
    );
    let min_z = position.z.saturating_sub(1);
    let max_z = position.z.saturating_add(1);

    Some((rect, (min_z, max_z)))
}

pub fn iter_nearby_occupied(ctx: &ViewContext) -> Vec<OccupiedTileV1> {
    let Some((rect, (min_z, max_z))) = find_ranges(ctx) else {
        return Vec::new();
    };

    let sec_min_x = rect.min.x / SECTOR_SIZE;
    let sec_max_x = rect.max.x / SECTOR_SIZE;
    let sec_min_y = rect.min.y / SECTOR_SIZE;
    let sec_max_y = rect.max.y / SECTOR_SIZE;

    let mut occupied = Vec::new();
    for z in min_z..=max_z {
        for sx in sec_min_x..=sec_max_x {
            for sy in sec_min_y..=sec_max_y {
                let sector_key = ((z as u64) << 32) | ((sx as u64) << 16) | (sy as u64);
                for tile in ctx.db.occupied_tile_v1().sector_key().filter(sector_key) {
                    let pos = Vec3::from_map_id(tile.map_id);
                    if pos.z == z && rect.contains(pos.into()) {
                        occupied.push(tile);
                    }
                }
            }
        }
    }
    occupied
}
