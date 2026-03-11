use crate::{
    constants::{DEFAULT_SPAWN_X, DEFAULT_SPAWN_Y},
    error::{ErrorMapper, ServiceError, ServiceResult},
    repository::{
        character::{character_v1, services::CharacterReducerContext},
        world::{
            CharacterPositionV1, OccupiedTileV1, occupied_tile_v1, offline_character_position_v1, online_character_position_v1,
            types::Vec2,
        },
    },
};
use spacetimedb::{Identity, ReducerContext, Table};
use std::ops::Deref;
use thiserror::Error;

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

    pub fn is_occupied(&self, pos: Vec2) -> bool {
        self.db.occupied_tile_v1().tile_id().find(pos.tile_id()).is_some()
    }

    fn occupy_tile(&self, pos: Vec2, character_id: u64) {
        if let Some(mut tile) = self.db.occupied_tile_v1().tile_id().find(pos.tile_id()) {
            if !tile.character_ids.contains(&character_id) {
                tile.character_ids.push(character_id);
                self.db.occupied_tile_v1().tile_id().update(tile);
            }
        } else {
            self.db.occupied_tile_v1().insert(OccupiedTileV1 {
                tile_id: pos.tile_id(),
                sector_key: pos.sector_key(),
                character_ids: vec![character_id],
            });
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
            .unwrap_or_else(|| self.next_available_position(character_id));

        self.occupy_tile(Vec2::new(position.x, position.y), character_id);
        self.db.offline_character_position_v1().character_id().delete(character_id);
        self.db
            .online_character_position_v1()
            .character_id()
            .insert_or_update(position);
    }

    pub fn despawn_character(&self, user_id: Identity) {
        for character in self.db.character_v1().user_id().filter(user_id) {
            let character_id = character.character_id;
            if let Some(position) = self.find_online_position(character_id) {
                self.db
                    .offline_character_position_v1()
                    .character_id()
                    .insert_or_update(position);
            }
            self.db.online_character_position_v1().character_id().delete(character_id);
        }
    }

    fn next_available_position(&self, character_id: u64) -> CharacterPositionV1 {
        let origin_x = u32::from(DEFAULT_SPAWN_X);
        let origin_y = u32::from(DEFAULT_SPAWN_Y);

        for distance in 0..=u32::from(u16::MAX) * 2 {
            for x_offset in (0..=distance).rev() {
                let y_offset = distance - x_offset;

                let Some(x) = origin_x.checked_add(x_offset).and_then(|value| u16::try_from(value).ok()) else {
                    continue;
                };
                let Some(y) = origin_y.checked_add(y_offset).and_then(|value| u16::try_from(value).ok()) else {
                    continue;
                };

                let position = Vec2::new(x, y);
                if !self.is_occupied(position) {
                    return CharacterPositionV1 { character_id, x, y };
                }
            }
        }

        unreachable!("No open spawn position available");
    }
}

#[derive(Debug, Error)]
enum WorldError {
    #[error("Character {0} has no position")]
    CharacterPositionNotFound(u64),
}

impl WorldError {
    fn character_position_not_found(character_id: u64) -> ServiceError {
        Self::CharacterPositionNotFound(character_id).map_not_found_error()
    }
}
