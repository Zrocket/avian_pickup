//! Shows a minimal example of using `avian_pickup` with Bevy.
//! Here, the actor has a collider as well.

use std::f32::consts::FRAC_PI_2;

use avian3d::prelude::*;
use avian_interpolation3d::prelude::*;
use avian_pickup::prelude::*;
use bevy::{
    app::RunFixedMainLoop, color::palettes::tailwind, input::mouse::MouseMotion, prelude::*, render::mesh, 
};

mod util;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default(),
            // Because we are moving the camera independently of the physics system,
            // interpolation is needed to prevent jittering.
            AvianInterpolationPlugin::default(),
            AvianPickupPlugin::default(),
            // This is just here to make the example look a bit nicer.
            util::plugin(util::Example::Generic),
        ))
        .add_systems(Startup, setup)
        // Input handling and camera movement need to be executed every frame,
        // so we run them in a variable timestep.
        // We also want them to happen before the physics system, so we add them
        // to the last variable timestep schedule before the fixed timestep systems run.
        .add_systems(
            RunFixedMainLoop,
            (handle_input, rotate_camera).in_set(RunFixedMainLoopSystem::BeforeFixedMainLoop),
        )
        .run();
}

#[derive(Debug, PhysicsLayer, Default)]
enum CollisionLayer {
    #[default]
    Default,
    Player,
    Prop,
}

/// Spawn the camera, light, ground, and a box to pick up.
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let terrain_material = materials.add(Color::WHITE);
    let prop_material = materials.add(Color::from(tailwind::EMERALD_300));

    commands.spawn((
        Name::new("Player Camera"),
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 1.0, 5.0),
            ..default()
        },
        // Add this to set up the camera as the entity that can pick up
        // objects.
        AvianPickupActor {
            prop_filter: SpatialQueryFilter::from_mask(CollisionLayer::Prop),
            actor_filter: SpatialQueryFilter::from_mask(CollisionLayer::Player),
            obstacle_filter: SpatialQueryFilter::from_mask(CollisionLayer::Default),
            hold: AvianPickupActorHoldConfig {
                // Make sure the prop is far enough away from
                // our collider when looking straight down
                pitch_range: -50.0_f32.to_radians()..=75.0_f32.to_radians(),
                ..default()
            },
            ..default()
        },
        CollisionLayers::new(CollisionLayer::Player, LayerMask::ALL),
        RigidBody::Kinematic,
        Collider::capsule(0.2, 1.3),
        // We are moving this entity manually, so disable interpolation.
        InterpolationMode::None,
    ));

    commands.spawn((
        Name::new("Light"),
        PointLightBundle {
            transform: Transform::from_xyz(3.0, 8.0, 3.0),
            point_light: PointLight {
                color: Color::WHITE,
                intensity: 2_000_000.0,
                shadows_enabled: true,
                ..default()
            },
            ..default()
        },
    ));

    let ground_shape = Cuboid::new(15.0, 0.25, 15.0);
    commands.spawn((
        Name::new("Ground"),
        Mesh3d(meshes.add(Mesh::from(ground_shape))),
        MeshMaterial3d(terrain_material.clone()),
        RigidBody::Static,
        Collider::from(ground_shape),
        CollisionLayers::new(CollisionLayer::Default, LayerMask::ALL),
    ));

    let box_shape = Cuboid::from_size(Vec3::splat(0.5));
    commands.spawn((
        Name::new("Box"),
        Mesh3d(meshes.add(Mesh::from(box_shape))),
        MeshMaterial3d(prop_material.clone()),
        Transform::from_xyz(0.0, 2.0, 3.5),
        // All `RigidBody::Dynamic` entities are able to be picked up.
        RigidBody::Dynamic,
        Mass(5.0),
        Collider::from(box_shape),
        CollisionLayers::new(CollisionLayer::Prop, LayerMask::ALL),
    ));
}

/// Pass player input along to `avian_pickup`
fn handle_input(
    mut avian_pickup_input_writer: EventWriter<AvianPickupInput>,
    key_input: Res<ButtonInput<MouseButton>>,
    actors: Query<Entity, With<AvianPickupActor>>,
) {
    for actor in &actors {
        if key_input.just_pressed(MouseButton::Left) {
            avian_pickup_input_writer.send(AvianPickupInput {
                action: AvianPickupAction::Throw,
                actor,
            });
        }
        if key_input.just_pressed(MouseButton::Right) {
            avian_pickup_input_writer.send(AvianPickupInput {
                action: AvianPickupAction::Drop,
                actor,
            });
        }
        if key_input.pressed(MouseButton::Right) {
            avian_pickup_input_writer.send(AvianPickupInput {
                action: AvianPickupAction::Pull,
                actor,
            });
        }
    }
}

fn rotate_camera(
    mut mouse_motion: EventReader<MouseMotion>,
    // Note how we change the `Rotation` and not the `Transform` as this is a rigid body now.
    mut cameras: Query<&mut Rotation, With<Camera>>,
) {
    for mut rotation in &mut cameras {
        // The factors are just arbitrary mouse sensitivity values.
        // It's often nicer to have a faster horizontal sensitivity than vertical.
        let mouse_sensitivity = Vec2::new(0.003, 0.002);

        for motion in mouse_motion.read() {
            let delta_yaw = -motion.delta.x * mouse_sensitivity.x;
            let delta_pitch = -motion.delta.y * mouse_sensitivity.y;

            const PITCH_LIMIT: f32 = FRAC_PI_2 - 0.01;
            let (yaw, pitch, roll) = rotation.to_euler(EulerRot::YXZ);
            let yaw = yaw + delta_yaw;
            let pitch = (pitch + delta_pitch).clamp(-PITCH_LIMIT, PITCH_LIMIT);
            rotation.0 = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
        }
    }
}
