use bevy::prelude::*;

use crate::{
    prelude::{AvianPickupActor, AvianPickupActorState},
    pull_object::PullObject,
};

pub(super) mod prelude {
    pub use super::AvianPickupEvent;
}

pub(super) fn plugin(app: &mut App) {
    app.register_type::<AvianPickupEvent>()
        .add_event::<AvianPickupEvent>()
        .observe(usher_event);
}

/// Event for picking up and throwing objects.
/// Send this to tell Avian Pickup to do its thing.
#[derive(Event, Debug, Clone, PartialEq, Eq, Reflect)]
#[reflect(Debug, PartialEq)]
#[cfg_attr(
    feature = "serialize",
    derive(serde::Serialize, serde::Deserialize),
    reflect(Serialize, Deserialize)
)]
pub enum AvianPickupEvent {
    /// The left mouse button was just pressed this update.
    JustPressedL,
    /// The right mouse button was just pressed this update.
    JustPressedR,
    /// The right mouse button was pressed.
    PressedR,
}

fn usher_event(
    trigger: Trigger<AvianPickupEvent>,
    mut commands: Commands,
    q_actor: Query<(
        Option<&AvianPickupActorState>,
        Has<AvianPickupActor>,
        Has<GlobalTransform>,
    )>,
) {
    let event = trigger.event();
    let entity = trigger.entity();
    // Unwrap cannot fail: the query only checks optional components
    let (state, has_actor, has_transform) = q_actor.get(entity).unwrap();
    let Some(&state) = state else {
        error!(
            "`AvianPickupEvent` was triggered on an entity without `AvianPickupActorState`. Ignoring."
        );
        return;
    };
    // Doing these checks to that other systems can just call `unwrap`
    if !has_actor {
        error!(
            "`AvianPickupEvent` was triggered on an entity without `AvianPickupActor`. Ignoring."
        );
        return;
    }
    if !has_transform {
        error!(
            "`AvianPickupEvent` was triggered on an entity without `GlobalTransform`. Ignoring."
        );
        return;
    }

    match event {
        AvianPickupEvent::JustPressedL => info!("Throw"),
        AvianPickupEvent::JustPressedR if state == AvianPickupActorState::Holding => info!("Drop"),
        AvianPickupEvent::JustPressedR | AvianPickupEvent::PressedR => {
            if state != AvianPickupActorState::Holding {
                commands.trigger_targets(PullObject, entity)
            }
        }
    }
}
