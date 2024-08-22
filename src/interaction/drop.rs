use crate::{prelude::*, verb::Dropping};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(PhysicsSchedule, drop.in_set(HandleVerbSystem::Drop));
}

/// DetachObject
fn drop(
    mut commands: Commands,
    mut q_actor: Query<(Entity, &mut AvianPickupActorState, &mut Cooldown, &Dropping)>,
    mut q_prop: Query<(
        &mut Mass,
        &mut LinearVelocity,
        &mut AngularVelocity,
        Option<&NonPickupMass>,
    )>,
) {
    for (actor, mut state, mut cooldown, drop) in q_actor.iter_mut() {
        let prop = drop.0;
        *state = AvianPickupActorState::Idle;
        commands.entity(prop).remove::<HeldProp>();
        cooldown.drop();
        commands.entity(actor).remove::<Dropping>();
        // Safety: the prop is a dynamic rigid body and thus is guaranteed to have a
        // mass, linvel, and angvel.
        let (mut mass, mut velocity, mut angvel, non_pickup_mass) = q_prop.get_mut(prop).unwrap();
        let Some(non_pickup_mass) = non_pickup_mass else {
            error!("Failed to give a dropped prop its pre-pickup mass back. Ignoring.");
            continue;
        };
        mass.0 = non_pickup_mass.0;
        velocity.0 = Vec3::ZERO;
        angvel.0 = Vec3::ZERO;
        // TODO: let the user know this prop was dropped through an event or
        // observer. Do events sent in a fixed timestep get propagated
        // to `PostUpdate` even when two fixed update loops passed?
    }
}
