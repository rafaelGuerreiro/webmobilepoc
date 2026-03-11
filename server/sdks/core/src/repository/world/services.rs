use crate::{
    constants::{
        DEFAULT_CHARACTER_SPEED, DEFAULT_SPAWN_X, DEFAULT_SPAWN_Y, MOVEMENT_COOLDOWN_FACTOR, MOVEMENT_INTENTION_WINDOW_MS,
        SECTOR_SIZE,
    },
    error::{ErrorMapper, ServiceError, ServiceResult},
    repository::{
        character::{character_v1, services::CharacterReducerContext},
        item::services::ItemReducerContext,
        world::{
            CharacterPositionV1, MapV1, MovementCooldownV1, OccupiedTileV1, OneshotMovementIntentionV1, WalkedMapChunkV1,
            map_v1, movement_cooldown_v1, occupied_tile_v1, offline_character_position_v1, oneshot_movement_intention_v1,
            online_character_position_v1,
            types::{DirectionV1, MapTileV1, MovementV1, Rect, Vec2, Vec3},
            walked_map_chunk_v1,
        },
    },
};
use spacetimedb::{Identity, ReducerContext, Table, Timestamp};
use std::{ops::Deref, time::Duration};
use thiserror::Error;
use webmobiledb_shared::constants::GROUND_LEVEL;

pub trait WorldReducerContext {
    fn world_services(&self) -> WorldServices<'_>;
}

impl WorldReducerContext for ReducerContext {
    fn world_services(&self) -> WorldServices<'_> {
        WorldServices { ctx: self }
    }
}

pub struct WorldServices<'a> {
    ctx: &'a ReducerContext,
}

impl Deref for WorldServices<'_> {
    type Target = ReducerContext;

    fn deref(&self) -> &Self::Target {
        self.ctx
    }
}

impl WorldServices<'_> {
    pub fn find_online_position(&self, character_id: u64) -> Option<CharacterPositionV1> {
        self.db.online_character_position_v1().character_id().find(character_id)
    }

    pub fn find_offline_position(&self, character_id: u64) -> Option<CharacterPositionV1> {
        self.db.offline_character_position_v1().character_id().find(character_id)
    }

    pub fn find_cooldown(&self, character_id: u64) -> Option<MovementCooldownV1> {
        self.db.movement_cooldown_v1().character_id().find(character_id)
    }

    pub fn is_movement_allowed(&self, character_id: u64) -> bool {
        if let Some(cooldown) = self.find_cooldown(character_id) {
            self.timestamp >= cooldown.can_move_at
        } else {
            true
        }
    }

    pub fn find_map_at(&self, pos: Vec3) -> Option<MapV1> {
        let point = Vec2::new(pos.x, pos.y);
        self.db
            .map_v1()
            .sector_key()
            .filter(pos.sector_key())
            .find(|chunk| chunk.z == pos.z && Rect::from(chunk).contains(point))
    }

    pub fn is_walkable(&self, position: &CharacterPositionV1, target: Vec3) -> bool {
        if let Some(cache) = self.db.walked_map_chunk_v1().character_id().find(position.character_id)
            && cache.z == target.z
            && Rect::from(&cache).contains(target.into())
        {
            return true;
        }

        let Some(chunk) = self.find_map_at(target) else {
            return false;
        };

        if !chunk.tile.is_walkable() {
            return false;
        }

        self.db
            .walked_map_chunk_v1()
            .character_id()
            .insert_or_update(WalkedMapChunkV1 {
                character_id: position.character_id,
                map_id: chunk.map_id,
                x1: chunk.x1,
                y1: chunk.y1,
                x2: chunk.x2,
                y2: chunk.y2,
                z: chunk.z,
            });

        true
    }

    pub fn is_occupied(&self, pos: Vec3) -> bool {
        self.db.occupied_tile_v1().map_id().find(pos.map_id()).is_some()
    }

    fn occupy_tile(&self, map_id: u64, character_id: u64) {
        if let Some(mut tile) = self.db.occupied_tile_v1().map_id().find(map_id) {
            if !tile.character_ids.contains(&character_id) {
                tile.character_ids.push(character_id);
                self.db.occupied_tile_v1().map_id().update(tile);
            }
        } else {
            self.db.occupied_tile_v1().insert(OccupiedTileV1 {
                map_id,
                sector_key: Vec3::from_map_id(map_id).sector_key(),
                character_ids: vec![character_id],
            });
        }
    }

    fn vacate_tile(&self, map_id: u64, character_id: u64) {
        let Some(mut tile) = self.db.occupied_tile_v1().map_id().find(map_id) else {
            return;
        };
        tile.character_ids.retain(|&id| id != character_id);
        if tile.character_ids.is_empty() {
            self.db.occupied_tile_v1().map_id().delete(map_id);
        } else {
            self.db.occupied_tile_v1().map_id().update(tile);
        }
    }

    pub fn get_online_position(&self, character_id: u64) -> ServiceResult<CharacterPositionV1> {
        self.find_online_position(character_id)
            .ok_or_else(|| WorldError::character_position_not_found(character_id))
    }

    pub fn get_offline_position(&self, character_id: u64) -> ServiceResult<CharacterPositionV1> {
        self.find_offline_position(character_id)
            .ok_or_else(|| WorldError::character_position_not_found(character_id))
    }

    pub fn spawn_character(&self, user_id: Identity) {
        self.despawn_character(user_id);

        let Ok(character) = self.character_services().get_current(user_id) else {
            return;
        };

        let character_id = character.character_id;
        let position = self
            .find_offline_position(character_id)
            .or_else(|| self.find_online_position(character_id))
            .unwrap_or_else(|| {
                let spawn = Vec3::new(DEFAULT_SPAWN_X, DEFAULT_SPAWN_Y, GROUND_LEVEL);
                CharacterPositionV1 {
                    character_id,
                    x: spawn.x,
                    y: spawn.y,
                    z: spawn.z,
                    movement: MovementV1::default(),
                    direction: DirectionV1::default(),
                    arrives_at: Timestamp::UNIX_EPOCH,
                }
            });

        let map_id = Vec3::new(position.x, position.y, position.z).map_id();
        self.db.offline_character_position_v1().character_id().delete(character_id);
        self.occupy_tile(map_id, character_id);
        self.db
            .online_character_position_v1()
            .character_id()
            .insert_or_update(position);
    }

    pub fn despawn_character(&self, user_id: Identity) {
        for character in self.db.character_v1().user_id().filter(user_id) {
            let character_id = character.character_id;
            self.item_services().close_all_containers(character_id);
            if let Some(position) = self.find_online_position(character_id) {
                let map_id = Vec3::new(position.x, position.y, position.z).map_id();
                self.vacate_tile(map_id, character_id);
                self.db
                    .offline_character_position_v1()
                    .character_id()
                    .insert_or_update(position);
            }
            self.db.online_character_position_v1().character_id().delete(character_id);
            self.db.movement_cooldown_v1().character_id().delete(character_id);
            self.db.oneshot_movement_intention_v1().character_id().delete(character_id);
            self.db.walked_map_chunk_v1().character_id().delete(character_id);
        }
    }

    pub fn seed_initial_map(&self) {
        let existing_count = self.db.map_v1().count();
        if existing_count > 0 {
            return;
        }

        let grass_start: u16 = 1024;
        let grass_end: u16 = grass_start + 256;
        let water_margin: u16 = 16;
        let edge_start = grass_start - water_margin;
        let edge_end = grass_end + water_margin;

        // Grass area
        self.insert_rect_chunks(
            Rect::new(grass_start, grass_start, grass_end, grass_end),
            GROUND_LEVEL,
            MapTileV1::Grass,
        );

        // Water margins: top
        self.insert_rect_chunks(
            Rect::new(edge_start, edge_start, edge_end, grass_start - 1),
            GROUND_LEVEL,
            MapTileV1::Water,
        );
        // Water margins: bottom
        self.insert_rect_chunks(
            Rect::new(edge_start, grass_end + 1, edge_end, edge_end),
            GROUND_LEVEL,
            MapTileV1::Water,
        );
        // Water margins: left
        self.insert_rect_chunks(
            Rect::new(edge_start, grass_start, grass_start - 1, grass_end),
            GROUND_LEVEL,
            MapTileV1::Water,
        );
        // Water margins: right
        self.insert_rect_chunks(
            Rect::new(grass_end + 1, grass_start, edge_end, grass_end),
            GROUND_LEVEL,
            MapTileV1::Water,
        );
    }

    fn insert_rect_chunks(&self, rect: Rect, z: u8, tile: MapTileV1) {
        if rect.min.x > rect.max.x || rect.min.y > rect.max.y {
            return;
        }

        let mut cx = rect.min.x;
        while cx <= rect.max.x {
            let sector_end_x = ((cx / SECTOR_SIZE) + 1) * SECTOR_SIZE - 1;
            let chunk_x2 = sector_end_x.min(rect.max.x);
            let mut cy = rect.min.y;
            while cy <= rect.max.y {
                let sector_end_y = ((cy / SECTOR_SIZE) + 1) * SECTOR_SIZE - 1;
                let chunk_y2 = sector_end_y.min(rect.max.y);
                let pos = Vec3::new(cx, cy, z);
                self.db.map_v1().insert(MapV1 {
                    map_id: pos.map_id(),
                    sector_key: pos.sector_key(),
                    x1: cx,
                    y1: cy,
                    x2: chunk_x2,
                    y2: chunk_y2,
                    z,
                    tile,
                });
                cy = chunk_y2 + 1;
            }
            cx = chunk_x2 + 1;
        }
    }

    pub fn move_character(&self, character_id: u64, movement: MovementV1) -> ServiceResult<()> {
        if let Some(cooldown) = self.find_cooldown(character_id)
            && self.timestamp < cooldown.can_move_at
        {
            let remaining = cooldown.can_move_at.duration_since(self.timestamp).unwrap_or_default();
            if remaining <= Duration::from_millis(MOVEMENT_INTENTION_WINDOW_MS) {
                self.schedule_movement_intention(character_id, movement, cooldown.can_move_at);
                return Ok(());
            }

            return Err(WorldError::movement_on_cooldown());
        }

        self.execute_movement(character_id, movement)
    }

    pub fn execute_movement_intention(&self, character_id: u64, movement: MovementV1) {
        if !self.is_movement_allowed(character_id) {
            return;
        }
        let _ = self.execute_movement(character_id, movement);
    }

    fn execute_movement(&self, character_id: u64, movement: MovementV1) -> ServiceResult<()> {
        let character = self.character_services().get_online(character_id)?;
        let position = self.get_online_position(character.character_id)?;

        let (target_x, target_y) = movement.translate(position.x, position.y);
        if target_x == position.x && target_y == position.y {
            return Err(WorldError::movement_out_of_bounds());
        }

        let target = Vec3::new(target_x, target_y, position.z);
        if self.is_occupied(target) {
            return Err(WorldError::tile_occupied());
        }

        if !self.is_walkable(&position, target) {
            return Err(WorldError::tile_not_walkable());
        }

        let arrives_at = self.compute_movement_arrives_at(character.character_id, movement);

        let current_map_id = Vec3::new(position.x, position.y, position.z).map_id();
        self.vacate_tile(current_map_id, position.character_id);
        self.occupy_tile(target.map_id(), position.character_id);

        self.db
            .online_character_position_v1()
            .character_id()
            .update(CharacterPositionV1 {
                character_id: position.character_id,
                x: target.x,
                y: target.y,
                z: target.z,
                movement,
                direction: movement.into(),
                arrives_at,
            });

        self.set_movement_cooldown(character.character_id, arrives_at);
        self.item_services().close_inaccessible_containers(character.character_id);
        Ok(())
    }

    fn schedule_movement_intention(&self, character_id: u64, movement: MovementV1, can_move_at: Timestamp) {
        self.db
            .oneshot_movement_intention_v1()
            .character_id()
            .insert_or_update(OneshotMovementIntentionV1 {
                character_id,
                scheduled_at: can_move_at.into(),
                movement,
            });
    }

    fn compute_movement_arrives_at(&self, character_id: u64, movement: MovementV1) -> Timestamp {
        let speed = self
            .character_services()
            .find_stats(character_id)
            .map(|s| s.speed as u64)
            .unwrap_or(DEFAULT_CHARACTER_SPEED as u64);

        let mut cooldown_ms = MOVEMENT_COOLDOWN_FACTOR / speed;
        if movement.is_diagonal() {
            cooldown_ms = cooldown_ms * 1_414_213 / 1_000_000;
        }
        self.timestamp + Duration::from_millis(cooldown_ms)
    }

    fn set_movement_cooldown(&self, character_id: u64, can_move_at: Timestamp) {
        self.db
            .movement_cooldown_v1()
            .character_id()
            .insert_or_update(MovementCooldownV1 {
                character_id,
                can_move_at,
            });
    }
}

#[derive(Debug, Error)]
enum WorldError {
    #[error("Character {0} has no position")]
    CharacterPositionNotFound(u64),

    #[error("Tile is not walkable")]
    TileNotWalkable,

    #[error("Tile is occupied by another player")]
    TileOccupied,

    #[error("Movement is on cooldown")]
    MovementOnCooldown,

    #[error("Movement is out of bounds")]
    MovementOutOfBounds,
}

impl WorldError {
    fn character_position_not_found(character_id: u64) -> ServiceError {
        Self::CharacterPositionNotFound(character_id).map_not_found_error()
    }

    fn tile_not_walkable() -> ServiceError {
        Self::TileNotWalkable.map_validation_error()
    }

    fn tile_occupied() -> ServiceError {
        Self::TileOccupied.map_validation_error()
    }

    fn movement_on_cooldown() -> ServiceError {
        Self::MovementOnCooldown.map_validation_error()
    }

    fn movement_out_of_bounds() -> ServiceError {
        Self::MovementOutOfBounds.map_validation_error()
    }
}
