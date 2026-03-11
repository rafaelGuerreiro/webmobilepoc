use spacetimedb::{Identity, Timestamp, table};

pub mod services;
pub mod views;

#[table(accessor = user_v1, private)]
pub struct UserV1 {
    #[primary_key]
    pub user_id: Identity,
    pub created_at: Timestamp,
    pub last_active_at: Timestamp,
}
