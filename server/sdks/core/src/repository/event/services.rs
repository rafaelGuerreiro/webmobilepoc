use crate::{
    error::ServiceResult,
    repository::{
        character::services::CharacterReducerContext,
        event::{
            OneshotDeferredEventV1, oneshot_deferred_event_v1,
            types::{DeferredEventV1, EventV1},
        },
        user::services::UserReducerContext,
        world::services::WorldReducerContext,
    },
};
use log::{info, warn};
use spacetimedb::{ReducerContext, Table};
use std::{ops::Deref, time::Duration};

pub trait EventReducerContext {
    fn event_services(&self) -> EventServices<'_>;

    fn publish(&self) -> EventPublisher<'_>;
}

impl EventReducerContext for ReducerContext {
    fn event_services(&self) -> EventServices<'_> {
        EventServices { ctx: self }
    }

    fn publish(&self) -> EventPublisher<'_> {
        EventPublisher { ctx: self }
    }
}

pub struct EventServices<'a> {
    ctx: &'a ReducerContext,
}

impl Deref for EventServices<'_> {
    type Target = ReducerContext;

    fn deref(&self) -> &Self::Target {
        self.ctx
    }
}

impl EventServices<'_> {
    fn handle_sync_event(&self, event: EventV1, _rethrow: bool) -> ServiceResult<()> {
        match event {
            EventV1::SystemInit => {
                self.world_services().seed_initial_map();
            },
            EventV1::UserCreated { .. } => {},
            EventV1::UserSignedIn { user_id } => {
                self.user_services().signed_in(user_id);
                self.character_services().clear_online_character(user_id);
                self.world_services().despawn_character(user_id);
            },
            EventV1::UserSignedOut { user_id } => {
                self.world_services().despawn_character(user_id);
                self.character_services().clear_online_character(user_id);
                self.user_services().signed_out(user_id);
            },
            EventV1::CharacterCreated { .. } => {},
            EventV1::CharacterSelected { user_id, .. } => {
                self.world_services().spawn_character(user_id);
            },
            EventV1::CharacterUnselected { user_id } => {
                self.world_services().despawn_character(user_id);
                self.character_services().clear_online_character(user_id);
            },
        }

        Ok(())
    }

    pub fn handle_deferred_event(&self, timer: OneshotDeferredEventV1) {
        match timer.event {
            DeferredEventV1::SignedOut { .. } => {},
        }
    }

    pub fn fire(&self, event: EventV1) -> ServiceResult<()> {
        if let Some(deferred) = event.into_deferred() {
            self.enqueue_deferred_event(deferred);
        }

        self.handle_sync_event(event, true)?;
        Ok(())
    }

    pub fn fire_and_forget(&self, event: EventV1) {
        if let Err(err) = self.handle_sync_event(event, false) {
            warn!(
                "Sync event handler failed: sender={}, event={event:?}, error={err}",
                self.sender()
            );
        }
    }

    fn enqueue_deferred_event(&self, event: DeferredEventV1) {
        // Schedule for 4 milliseconds later to allow sync handlers to complete, this is 250fps.
        let scheduled_at = self.timestamp + Duration::from_millis(4);

        let job = self.db.oneshot_deferred_event_v1().insert(OneshotDeferredEventV1 {
            job_id: 0,
            scheduled_at: scheduled_at.into(),
            event,
            sender: self.sender(),
            created_at: self.timestamp,
        });

        info!(
            "Queued deferred event: job_id={}, sender={}, event={:?}",
            job.job_id, job.sender, job.event
        );
    }

    #[allow(dead_code)]
    fn catch<F>(&self, event: &EventV1, rethrow: bool, function: F) -> ServiceResult<()>
    where
        F: FnOnce() -> ServiceResult<()>,
    {
        if let Err(e) = function() {
            if rethrow {
                return Err(e);
            }

            warn!("Error in event: {e}; sender={}, event={event:?}", self.sender());
        }

        Ok(())
    }
}

pub struct EventPublisher<'a> {
    ctx: &'a ReducerContext,
}

impl Deref for EventPublisher<'_> {
    type Target = ReducerContext;

    fn deref(&self) -> &Self::Target {
        self.ctx
    }
}
