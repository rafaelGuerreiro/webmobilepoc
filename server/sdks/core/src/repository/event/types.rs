use crate::{
    error::ServiceResult,
    repository::event::services::{EventPublisher, EventReducerContext},
};
use spacetimedb::{Identity, SpacetimeType};

#[derive(Debug, Clone, Copy)]
pub enum EventV1 {
    SystemInit,
    UserSignedIn { user_id: Identity },
    UserSignedOut { user_id: Identity },
}

#[derive(Debug, Clone, Copy, SpacetimeType)]
pub enum DeferredEventV1 {
    SignedOut { user_id: Identity },
}

impl EventV1 {
    pub fn into_deferred(self) -> Option<DeferredEventV1> {
        match self {
            Self::UserSignedOut { user_id } => Some(DeferredEventV1::SignedOut { user_id }),
            _ => None,
        }
    }
}

impl EventPublisher<'_> {
    pub fn system_init(&self) {
        self.event_services().fire_and_forget(EventV1::SystemInit);
    }

    pub fn user_signed_in(&self, user_id: Identity) -> ServiceResult<()> {
        self.event_services().fire(EventV1::UserSignedIn { user_id })
    }

    pub fn user_signed_out(&self, user_id: Identity) {
        self.event_services().fire_and_forget(EventV1::UserSignedOut { user_id })
    }
}
