use crate::{
    constants::{DEFAULT_SPAWN_X, DEFAULT_SPAWN_Y},
    error::{ErrorMapper, ServiceError, ServiceResult},
    repository::world::{
        OccupiedTileV1, UserPositionV1, occupied_tile_v1, offline_user_position_v1, online_user_position_v1, types::Vec2,
    },
};
use spacetimedb::{Identity, ReducerContext};
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
    pub fn find_online_position(&self, user_id: Identity) -> Option<UserPositionV1> {
        self.db.online_user_position_v1().user_id().find(user_id)
    }

    pub fn find_offline_position(&self, user_id: Identity) -> Option<UserPositionV1> {
        self.db.offline_user_position_v1().user_id().find(user_id)
    }

    pub fn is_occupied(&self, pos: Vec2) -> bool {
        self.db.occupied_tile_v1().tile_id().find(pos.tile_id()).is_some()
    }

    fn occupy_tile(&self, pos: Vec2, user_id: Identity) {
        self.db.occupied_tile_v1().tile_id().insert_or_update(OccupiedTileV1 {
            tile_id: pos.tile_id(),
            sector_key: pos.sector_key(),
            user_id,
        });
    }

    pub fn get_online_position(&self, user_id: Identity) -> ServiceResult<UserPositionV1> {
        self.find_online_position(user_id)
            .ok_or_else(|| WorldError::user_position_not_found(user_id))
    }

    pub fn spawn_user(&self, user_id: Identity) {
        let position = self
            .find_offline_position(user_id)
            .or_else(|| self.find_online_position(user_id))
            .unwrap_or_else(|| self.next_available_position(user_id));

        self.occupy_tile(Vec2::new(position.x, position.y), user_id);
        self.db.offline_user_position_v1().user_id().delete(user_id);
        self.db.online_user_position_v1().user_id().insert_or_update(position);
    }

    pub fn despawn_user(&self, user_id: Identity) {
        if let Some(position) = self.find_online_position(user_id) {
            self.db.offline_user_position_v1().user_id().insert_or_update(position);
        }
        self.db.online_user_position_v1().user_id().delete(user_id);
    }

    fn next_available_position(&self, user_id: Identity) -> UserPositionV1 {
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
                    return UserPositionV1 { user_id, x, y };
                }
            }
        }

        unreachable!("No open spawn position available");
    }
}

#[derive(Debug, Error)]
enum WorldError {
    #[error("User {0} has no position")]
    UserPositionNotFound(Identity),
}

impl WorldError {
    fn user_position_not_found(user_id: Identity) -> ServiceError {
        Self::UserPositionNotFound(user_id).map_not_found_error()
    }
}
