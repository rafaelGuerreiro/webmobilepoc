use crate::repository::user::{UserV1, user_v1__view};
use spacetimedb::{ViewContext, view};

#[view(accessor = vw_user_me_v1, public)]
pub fn vw_user_me_v1(ctx: &ViewContext) -> Option<UserV1> {
    ctx.db.user_v1().user_id().find(ctx.sender())
}
