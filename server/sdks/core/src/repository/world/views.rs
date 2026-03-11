use crate::{
    extend::proximity::iter_nearby_occupied,
    repository::world::{UserPositionV1, online_user_position_v1__view},
};
use spacetimedb::{ViewContext, view};

#[view(accessor = vw_world_my_position_v1, public)]
pub fn vw_world_my_position_v1(ctx: &ViewContext) -> Option<UserPositionV1> {
    ctx.db.online_user_position_v1().user_id().find(ctx.sender())
}

#[view(accessor = vw_nearby_positions_v1, public)]
pub fn vw_nearby_positions_v1(ctx: &ViewContext) -> Vec<UserPositionV1> {
    iter_nearby_occupied(ctx)
        .into_iter()
        .filter_map(|occupied| ctx.db.online_user_position_v1().user_id().find(occupied.user_id))
        .collect()
}
