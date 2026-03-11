use spacetimedb::{Identity, table};

pub mod services;
pub mod types;
pub mod views;

#[table(accessor = online_user_position_v1, private)]
#[table(accessor = offline_user_position_v1, private)]
pub struct UserPositionV1 {
    #[primary_key]
    pub user_id: Identity,
    pub x: u16,
    pub y: u16,
}

#[table(accessor = occupied_tile_v1, private)]
pub struct OccupiedTileV1 {
    #[primary_key]
    pub tile_id: u64,
    #[index(btree)]
    pub sector_key: u64,
    pub user_id: Identity,
}
