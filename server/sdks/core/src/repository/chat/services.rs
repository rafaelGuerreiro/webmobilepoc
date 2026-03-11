use crate::{
    constants::{CHAT_MESSAGE_MAX_LEN, CHAT_MESSAGE_MIN_LEN},
    error::{ErrorMapper, ServiceError, ServiceResult},
    extend::validate::ReducerContextRequirements,
    repository::{
        character::services::CharacterReducerContext,
        chat::{ChatBubbleV1, chat_bubble_v1},
        world::services::WorldReducerContext,
    },
};
use spacetimedb::{ReducerContext, Table};
use std::ops::Deref;
use thiserror::Error;

pub trait ChatReducerContext {
    fn chat_services(&self) -> ChatServices<'_>;
}

impl ChatReducerContext for ReducerContext {
    fn chat_services(&self) -> ChatServices<'_> {
        ChatServices { ctx: self }
    }
}

pub struct ChatServices<'a> {
    ctx: &'a ReducerContext,
}

impl Deref for ChatServices<'_> {
    type Target = ReducerContext;

    fn deref(&self) -> &Self::Target {
        self.ctx
    }
}

impl ChatServices<'_> {
    pub fn send_message(&self, character_id: u64, content: String) -> ServiceResult<()> {
        let content = content.trim();
        if content.is_empty() {
            return Err(ChatError::message_empty());
        }
        self.validate_str(content, "message", CHAT_MESSAGE_MIN_LEN as u64, CHAT_MESSAGE_MAX_LEN as u64)?;

        let position = self.world_services().get_online_position(character_id)?;
        let character = self.character_services().get_online(character_id)?;
        let stats = self.character_services().get_stats(character_id)?;

        self.db.chat_bubble_v1().insert(ChatBubbleV1 {
            bubble_id: 0,
            character_name: character.display_name,
            character_level: stats.level,
            content: content.to_string(),
            x: position.x,
            y: position.y,
            sent_at: self.timestamp,
        });
        Ok(())
    }
}

#[derive(Debug, Error)]
enum ChatError {
    #[error("Chat message cannot be empty")]
    MessageEmpty,
}

impl ChatError {
    fn message_empty() -> ServiceError {
        Self::MessageEmpty.map_validation_error()
    }
}
