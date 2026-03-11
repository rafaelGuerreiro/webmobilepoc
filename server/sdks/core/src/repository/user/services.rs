use crate::{
    error::{ErrorMapper, ServiceError, ServiceResult},
    repository::user::{UserV1, user_v1},
};
use spacetimedb::{Identity, ReducerContext};
use std::ops::Deref;
use thiserror::Error;

pub trait UserReducerContext {
    fn user_services(&self) -> UserServices<'_>;
}

impl UserReducerContext for ReducerContext {
    fn user_services(&self) -> UserServices<'_> {
        UserServices { ctx: self }
    }
}

pub struct UserServices<'a> {
    ctx: &'a ReducerContext,
}

impl Deref for UserServices<'_> {
    type Target = ReducerContext;

    fn deref(&self) -> &Self::Target {
        self.ctx
    }
}

impl UserServices<'_> {
    pub fn find(&self, user_id: Identity) -> Option<UserV1> {
        self.db.user_v1().user_id().find(user_id)
    }

    pub fn get(&self, user_id: Identity) -> ServiceResult<UserV1> {
        self.find(user_id).ok_or_else(|| UserError::user_not_found(user_id))
    }

    pub fn signed_in(&self, user_id: Identity) {
        let user = match self.find(user_id) {
            Some(mut user) => {
                user.last_active_at = self.timestamp;
                user
            },
            None => UserV1 {
                user_id,
                created_at: self.timestamp,
                last_active_at: self.timestamp,
            },
        };

        self.db.user_v1().user_id().insert_or_update(user);
    }

    pub fn signed_out(&self, user_id: Identity) {
        if let Some(mut user) = self.find(user_id) {
            user.last_active_at = self.timestamp;
            self.db.user_v1().user_id().update(user);
        }
    }
}

#[derive(Debug, Error)]
enum UserError {
    #[error("User {0} was not found")]
    UserNotFound(Identity),
}

impl UserError {
    fn user_not_found(user_id: Identity) -> ServiceError {
        Self::UserNotFound(user_id).map_not_found_error()
    }
}
