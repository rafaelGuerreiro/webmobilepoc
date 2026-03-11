use crate::{
    error::ServiceResult, extend::validate::ReducerContextRequirements, repository::chat::services::ChatReducerContext,
};
use spacetimedb::{ReducerContext, reducer};

#[reducer]
pub fn say_v1(ctx: &ReducerContext, content: String) -> ServiceResult<()> {
    let character = ctx.require_online()?;
    ctx.chat_services().send_message(character.character_id, content)?;
    Ok(())
}
