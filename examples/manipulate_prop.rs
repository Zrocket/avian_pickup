use std::f32::consts::FRAC_PI_2;

use avian3d::prelude::*;
use avian_pickup::{
    prelude::*,
    prop::{PreferredPickupDistanceOverride, PreferredPickupRotation},
};
use bevy::{
    app::RunFixedMainLoop,
    color::palettes::tailwind,
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    time::run_fixed_main_schedule,
};
use bevy_transform_interpolation::*;

mod util;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default(),
            // Because we are moving the camera independently of the physics system,
            // interpolation is needed to prevent jittering.
            TransformInterpolationPlugin::interpolate_all(),
            AvianPickupPlugin::default(),
            // This is just here to make the example look a bit nicer.
            util::plugin(util::Example::Manipulation),
        ))
        .add_systems(Startup, setup)
        // Need to read input and rotate camera before physics,
        // this is unfortunately the best way to schedule this:
        // <https://github.com/bevyengine/bevy/issues/14873>
        .add_systems(
            RunFixedMainLoop,
            (accumulate_input, handle_pickup_input, rotate_camera)
                .chain()
                .before(run_fixed_main_schedule),
        )
        .add_systems(FixedUpdate, move_prop)
        .run();
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
            interaction_distance: 15.0,
            ..default()
        },
        // This entity is moved in a variable timestep, so no interpolation is needed.
        NoRotationInterpolation,
        InputAccumulation::default(),
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
        PbrBundle {
            mesh: meshes.add(Mesh::from(ground_shape)),
            material: terrain_material.clone(),
            ..default()
        },
        RigidBody::Static,
        Collider::from(ground_shape),
    ));

    let box_shape = Cuboid::from_size(Vec3::splat(0.5));
    commands.spawn((
        Name::new("Box"),
        PbrBundle {
            mesh: meshes.add(Mesh::from(box_shape)),
            material: prop_material.clone(),
            transform: Transform::from_xyz(0.0, 2.0, 1.5),
            ..default()
        },
        // All `RigidBody::Dynamic` entities are able to be picked up.
        RigidBody::Dynamic,
        Collider::from(box_shape),
        PreferredPickupDistanceOverride::default(),
        PreferredPickupRotation::default(),
    ));
}

/// Pass player input along to `avian_pickup`
fn handle_pickup_input(
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
    time: Res<Time>,
    mut cameras: Query<(&mut Transform, &mut InputAccumulation), With<Camera>>,
) {
    for (mut transform, mut input) in &mut cameras {
        let dt = time.delta_seconds();
        if input.shift {
            continue;
        }

        let delta_yaw = -input.rotation.x * dt;
        let delta_pitch = -input.rotation.y * dt;
        input.rotation = Vec2::ZERO;

        // Add yaw (global)
        transform.rotate_y(delta_yaw);

        // Add pitch (local)
        const PITCH_LIMIT: f32 = FRAC_PI_2 - 0.01;
        let (yaw, pitch, roll) = transform.rotation.to_euler(EulerRot::YXZ);
        let pitch = (pitch + delta_pitch).clamp(-PITCH_LIMIT, PITCH_LIMIT);
        transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
    }
}

fn accumulate_input(
    mut mouse_motion: EventReader<MouseMotion>,
    mut mouse_wheel: EventReader<MouseWheel>,
    key_input: Res<ButtonInput<KeyCode>>,
    mut accumulation: Query<&mut InputAccumulation>,
) {
    for motion in mouse_motion.read() {
        for mut input in &mut accumulation {
            // The factors are just arbitrary mouse sensitivity values.
            // It's often nicer to have a faster horizontal sensitivity than vertical.
            let mouse_sensitivity = Vec2::new(0.12, 0.10);
            input.rotation += motion.delta * mouse_sensitivity;
        }
    }
    for wheel in mouse_wheel.read() {
        for mut input in &mut accumulation {
            const SCROLL_SENSITIVITY: f32 = 1.0;
            let delta = wheel.y * SCROLL_SENSITIVITY;
            input.zoom += delta as i32;
        }
    }
    for mut input in &mut accumulation {
        input.shift =
            key_input.pressed(KeyCode::ShiftLeft) || key_input.pressed(KeyCode::ShiftRight);
    }
}

/// Systems in fixed timesteps may not run every frame,
/// so we accumulate all input that happened since the last fixed update.
#[derive(Debug, Component, Default)]
struct InputAccumulation {
    zoom: i32,
    rotation: Vec2,
    shift: bool,
}

fn move_prop(
    time: Res<Time>,
    mut actors: Query<(&mut InputAccumulation, &AvianPickupActorState)>,
    mut props: Query<(
        &mut PreferredPickupDistanceOverride,
        &mut PreferredPickupRotation,
    )>,
) {
    let dt = time.delta_seconds();
    for (mut input, state) in &mut actors {
        let AvianPickupActorState::Holding(prop) = state else {
            continue;
        };
        let (mut distance, mut rotation) = props.get_mut(*prop).unwrap();
        const SCROLL_VELOCITY: f32 = 5.0;
        let delta = input.zoom as f32 * SCROLL_VELOCITY * dt;
        input.zoom = 0;

        distance.0 += delta;
        distance.0 = distance.0.clamp(0.5, 15.0);

        if !input.shift {
            continue;
        }
        let y_rotation_global = Quat::from_rotation_y(input.rotation.x * dt);
        let x_rotation_global = Quat::from_rotation_x(input.rotation.y * dt);
        rotation.0 = x_rotation_global * y_rotation_global * rotation.0;

        input.rotation = Vec2::ZERO;
    }
}