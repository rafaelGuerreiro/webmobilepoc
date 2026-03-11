use self::{
    services::WorldReducerContext,
    types::{DirectionV1, MapTileV1},
};
use crate::{error::ServiceResult, extend::validate::ReducerContextRequirements, repository::world::types::MovementV1};
use spacetimedb::{ReducerContext, ScheduleAt, Timestamp, reducer, table};

pub mod reducers;
pub mod services;
pub mod types;
pub mod views;

#[table(accessor = map_v1, private)]
pub struct MapV1 {
    #[primary_key]
    pub map_id: u64,
    #[index(btree)]
    pub sector_key: u64,
    pub x1: u16,
    pub y1: u16,
    pub x2: u16,
    pub y2: u16,
    pub z: u8,
    pub tile: MapTileV1,
}

#[table(accessor = town_temple_v1, private)]
pub struct TownTempleV1 {
    #[auto_inc]
    #[primary_key]
    pub town_temple_id: u64,
    pub name: String,
    pub x: u16,
    pub y: u16,
    pub z: u8,
}

#[table(accessor = movement_cooldown_v1, private)]
pub struct MovementCooldownV1 {
    #[primary_key]
    pub character_id: u64,
    pub can_move_at: Timestamp,
}

#[table(accessor = online_character_position_v1, private)]
#[table(accessor = offline_character_position_v1, private)]
pub struct CharacterPositionV1 {
    #[primary_key]
    pub character_id: u64,
    pub x: u16,
    pub y: u16,
    pub z: u8,
    pub movement: MovementV1,
    pub direction: DirectionV1,
    pub arrives_at: Timestamp,
}

#[table(accessor = walked_map_chunk_v1, private)]
pub struct WalkedMapChunkV1 {
    #[primary_key]
    pub character_id: u64,
    pub map_id: u64,
    pub x1: u16,
    pub y1: u16,
    pub x2: u16,
    pub y2: u16,
    pub z: u8,
}

#[table(accessor = occupied_tile_v1, private)]
pub struct OccupiedTileV1 {
    #[primary_key]
    pub map_id: u64,
    #[index(btree)]
    pub sector_key: u64,
    pub character_ids: Vec<u64>,
}

#[table(accessor = oneshot_movement_intention_v1, private, scheduled(oneshot_movement_intention_scheduled_v1))]
pub struct OneshotMovementIntentionV1 {
    #[primary_key]
    pub character_id: u64,
    pub scheduled_at: ScheduleAt,
    pub movement: MovementV1,
}

#[reducer]
pub fn oneshot_movement_intention_scheduled_v1(ctx: &ReducerContext, timer: OneshotMovementIntentionV1) -> ServiceResult<()> {
    ctx.require_internal_access()?;
    ctx.world_services()
        .execute_movement_intention(timer.character_id, timer.movement);
    Ok(())
}
