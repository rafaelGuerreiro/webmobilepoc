use crate::repository::user::{UserV1, user_v1};
use spacetimedb::{Identity, ReducerContext};
use std::ops::Deref;

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
    pub fn signed_in(&self, user_id: Identity) {
        let user = self.db.user_v1().user_id().find(user_id);
        let user = match user {
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
        if let Some(mut user) = self.db.user_v1().user_id().find(user_id) {
            user.last_active_at = self.timestamp;
            self.db.user_v1().user_id().update(user);
        }
    }
}
