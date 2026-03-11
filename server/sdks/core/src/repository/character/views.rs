use crate::repository::character::{
    CharacterStatsV1, CharacterV1, character_stats_v1__query, character_stats_v1__view, character_v1__query,
    character_v1__view, online_character_v1__view,
};
use spacetimedb::{RawQuery, ViewContext, view};

#[view(accessor = vw_character_me_v1, public)]
pub fn vw_character_me_v1(ctx: &ViewContext) -> Option<CharacterV1> {
    let current = ctx.db.online_character_v1().user_id().find(ctx.sender())?;
    ctx.db.character_v1().character_id().find(current.character_id)
}

#[view(accessor = vw_character_me_stats_v1, public)]
pub fn vw_character_me_stats_v1(ctx: &ViewContext) -> Option<CharacterStatsV1> {
    let current = ctx.db.online_character_v1().user_id().find(ctx.sender())?;
    ctx.db.character_stats_v1().character_id().find(current.character_id)
}

#[view(accessor = vw_character_all_mine_v1, public)]
pub fn vw_character_all_mine_v1(ctx: &ViewContext) -> RawQuery<CharacterV1> {
    ctx.from.character_v1().r#where(|c| c.user_id.eq(ctx.sender())).build()
}

#[view(accessor = vw_character_all_mine_stats_v1, public)]
pub fn vw_character_all_mine_stats_v1(ctx: &ViewContext) -> RawQuery<CharacterStatsV1> {
    ctx.from.character_stats_v1().r#where(|c| c.user_id.eq(ctx.sender())).build()
}
