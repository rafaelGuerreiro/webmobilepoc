pub mod constants;
pub mod error;
pub mod extend;
pub mod repository;

use spacetimedb::ReducerContext;

pub use error::ServiceResult;

pub fn init(ctx: &ReducerContext) {
    repository::init(ctx);
}

pub fn identity_connected(ctx: &ReducerContext) -> ServiceResult<()> {
    repository::identity_connected(ctx)
}

pub fn identity_disconnected(ctx: &ReducerContext) {
    repository::identity_disconnected(ctx);
}
