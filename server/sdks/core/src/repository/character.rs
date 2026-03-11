use self::types::{ClassV1, RaceV1};
use crate::repository::character::types::GenderV1;
use spacetimedb::{Identity, Timestamp, table};

pub mod reducers;
pub mod services;
pub mod types;
pub mod views;

#[table(accessor = character_v1, private)]
pub struct CharacterV1 {
    #[auto_inc]
    #[primary_key]
    pub character_id: u64,
    #[index(btree)]
    pub user_id: Identity,
    #[unique]
    pub name: String,
    pub display_name: String,
    pub race: RaceV1,
    pub class: ClassV1,
    pub gender: GenderV1,
    pub created_at: Timestamp,
}

#[table(accessor = character_stats_v1, private)]
pub struct CharacterStatsV1 {
    #[primary_key]
    pub character_id: u64,
    #[index(btree)]
    pub user_id: Identity,
    pub level: u16,
    pub experience: u64,
    pub health: u32,
    pub mana: u32,
    pub capacity: u32,
    pub free_capacity: u32,
    pub speed: u16,
    pub attack_speed: u16,
}

#[table(accessor = online_character_v1, private)]
pub struct OnlineCharacterV1 {
    #[primary_key]
    pub user_id: Identity,
    pub character_id: u64,
    pub signed_in_at: Timestamp,
}
