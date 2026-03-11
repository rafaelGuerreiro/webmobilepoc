use spacetimedb::{Identity, Timestamp, table};

pub mod reducers;
pub mod services;
pub mod views;

/// Event table: rows are broadcast to all subscribers and auto-deleted.
#[table(accessor = chat_bubble_v1, public, event)]
pub struct ChatBubbleV1 {
    #[auto_inc]
    #[primary_key]
    pub bubble_id: u64,
    pub user_id: Identity,
    pub content: String,
    pub x: u16,
    pub y: u16,
    pub sent_at: Timestamp,
}
