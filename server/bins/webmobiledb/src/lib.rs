use spacetimedb::{CaseConversionPolicy, ReducerContext, reducer};
use webmobiledb_core::ServiceResult;

#[spacetimedb::settings]
const CASE_CONVERSION_POLICY: CaseConversionPolicy = CaseConversionPolicy::None;

#[reducer(init)]
pub fn init(ctx: &ReducerContext) {
    webmobiledb_core::init(ctx);
}

#[reducer(client_connected)]
pub fn identity_connected(ctx: &ReducerContext) -> ServiceResult<()> {
    webmobiledb_core::identity_connected(ctx)
}

#[reducer(client_disconnected)]
pub fn identity_disconnected(ctx: &ReducerContext) {
    webmobiledb_core::identity_disconnected(ctx);
}
