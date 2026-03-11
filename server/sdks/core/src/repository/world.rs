use spacetimedb::table;

pub mod reducers;
pub mod services;
pub mod types;
pub mod views;

#[table(accessor = online_character_position_v1, private)]
#[table(accessor = offline_character_position_v1, private)]
pub struct CharacterPositionV1 {
    #[primary_key]
    pub character_id: u64,
    pub x: u16,
    pub y: u16,
}

#[table(accessor = occupied_tile_v1, private)]
pub struct OccupiedTileV1 {
    #[primary_key]
    pub tile_id: u64,
    #[index(btree)]
    pub sector_key: u64,
    pub character_ids: Vec<u64>,
}
