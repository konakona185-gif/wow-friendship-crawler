
use std::time::Duration;

use avian3d::prelude::{
    Collider,
    MoveAndSlideConfig,
    MoveAndSlideHitResponse,
};
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;

use benilla_assets::coords::bevy_to_wow;
use benilla_assets::WorldAssets;
use benilla_world::collision::WorldCollision;
use benilla_world::schedule::WorldStage;
use benilla_world::terrain_stream::{
    ViewFocus,
    WorldLoadProgress,
};
use benilla_world::view::Viewer;

const PLAYER_CAPSULE_RADIUS: f32 = 1.0 / 3.0;
const PLAYER_CAPSULE_HEIGHT: f32 = 2.027_777_7;
const PLAYER_SKIN_WIDTH: f32 = 0.02;
const GROUND_NORMAL_THRESHOLD: f32 = 0.642_787_64;
const GROUND_PROBE_LIFT: f32 = 0.1;
const GROUND_PROBE_DISTANCE: f32 = 0.3;

#[derive(Component)]
pub struct Character;

#[derive(Component)]
pub struct LocalPlayer;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PlayerId(pub u64);

#[derive(Component, Default)]
pub struct CharacterCommand {
    pub movement: Vec2,
    pub jump: bool,
    pub look_delta: f32,
    pub select_target: bool,
    pub ability_1: bool,
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CharacterInputSet {
    Collect,
    Process,
}

#[derive(Component)]
pub enum FellowshipRole {
    Tank,
    Healer,
    Dps,
}

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(Component)]
pub struct MovementSpeed(pub f32);

#[derive(Component)]
pub struct Velocity(pub Vec3);

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (CharacterInputSet::Collect, CharacterInputSet::Process).chain(),
        )
        .add_systems(
            Update,
            local_player_command_system.in_set(CharacterInputSet::Collect),
        )
        .add_systems(
            Update,
            player_movement_system
                .in_set(CharacterInputSet::Process)
                .in_set(WorldStage::Input),
        );
    }
}

fn local_player_command_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut query: Query<&mut CharacterCommand, With<LocalPlayer>>,
) {
    let mut forward_move = 0.0;
    let mut strafe_move = 0.0;

    if keyboard_input.pressed(KeyCode::KeyW)
        || keyboard_input.pressed(KeyCode::ArrowUp)
    {
        forward_move += 1.0;
    }

    if keyboard_input.pressed(KeyCode::KeyS)
        || keyboard_input.pressed(KeyCode::ArrowDown)
    {
        forward_move -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::KeyA)
        || keyboard_input.pressed(KeyCode::ArrowLeft)
    {
        strafe_move -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::KeyD)
        || keyboard_input.pressed(KeyCode::ArrowRight)
    {
        strafe_move += 1.0;
    }

    let mouse_delta = if mouse_button.pressed(MouseButton::Right) {
        mouse_motion.delta.x
    } else {
        0.0
    };

    for mut command in &mut query {
        command.movement = Vec2::new(strafe_move, forward_move);
        command.jump = keyboard_input.just_pressed(KeyCode::Space);
        command.look_delta = mouse_delta;
        command.select_target = keyboard_input.just_pressed(KeyCode::Tab);
        command.ability_1 = keyboard_input.just_pressed(KeyCode::Digit1);
    }
}

fn player_movement_system(
    time: Res<Time>,
    world: Option<WorldCollision<'_, '_>>,
    world_assets: Option<Res<WorldAssets>>,
    world_progress: Option<Res<WorldLoadProgress>>,
    mut view_focus: Option<ResMut<ViewFocus>>,
    mut viewer: Option<ResMut<Viewer>>,
    mut query: Query<
        (
            &mut Transform,
            &MovementSpeed,
            &mut Velocity,
            &mut CharacterCommand,
        ),
        With<Character>,
    >,
) {
    let gravity = -25.0;
    let jump_force = 9.0;
    let delta = time.delta_secs();

    if delta <= 0.0 {
        return;
    }

    let duration = Duration::from_secs_f32(delta);
    let capsule = Collider::capsule(
        PLAYER_CAPSULE_RADIUS,
        PLAYER_CAPSULE_HEIGHT - 2.0 * PLAYER_CAPSULE_RADIUS,
    );

    let has_world_assets = world_assets.is_some();

    let world_ready = has_world_assets
        && world.is_some()
        && world_progress.as_deref().is_none_or(|progress| {
            progress.focus_resident && progress.presentable()
        });

    for (mut transform, speed, mut velocity, mut command) in &mut query {
        if command.look_delta != 0.0 {
            let sensitivity = 0.003;
            transform.rotate_y(-command.look_delta * sensitivity);
        }

        command.look_delta = 0.0;

        let movement_input = command.movement;
        let jump_requested = command.jump;

        command.jump = false;

        let mut move_direction = Vec3::ZERO;

        if movement_input != Vec2::ZERO {
            let forward = transform.forward().as_vec3();
            let right = transform.right().as_vec3();

            move_direction =
                (forward * movement_input.y)
                + (right * movement_input.x);

            move_direction.y = 0.0;

            move_direction = move_direction.normalize_or_zero();
        }

        let horizontal_velocity = move_direction * speed.0;

        if !has_world_assets {
            transform.translation += horizontal_velocity * delta;
        } else if let Some(world) = world.as_ref().filter(|_| world_ready) {
            let mut grounded = false;

            if velocity.0.y <= 0.0 {
                if let Some(hit) = world.cast_mover(
                    &capsule,
                    transform.translation + Vec3::Y * GROUND_PROBE_LIFT,
                    Vec3::NEG_Y * GROUND_PROBE_DISTANCE,
                    PLAYER_SKIN_WIDTH,
                ) {
                    if hit.face_normal.y >= GROUND_NORMAL_THRESHOLD {
                        transform.translation.y +=
                            GROUND_PROBE_LIFT - hit.distance;
                        velocity.0.y = 0.0;
                        grounded = true;
                    }
                }
            }

            if jump_requested && grounded {
                velocity.0.y = jump_force;
                grounded = false;
            } else if !grounded {
                velocity.0.y += gravity * delta;
            } else {
                velocity.0.y = 0.0;
            }

            let movement_velocity =
                horizontal_velocity + Vec3::Y * velocity.0.y;

            let start_position = transform.translation;
            let end_position =
                start_position + movement_velocity * delta;

            let start_wow = bevy_to_wow(start_position);
            let end_wow = bevy_to_wow(end_position);

            let min = [
                start_wow[0].min(end_wow[0]) - PLAYER_CAPSULE_RADIUS,
                start_wow[1].min(end_wow[1]) - PLAYER_CAPSULE_RADIUS,
            ];

            let max = [
                start_wow[0].max(end_wow[0]) + PLAYER_CAPSULE_RADIUS,
                start_wow[1].max(end_wow[1]) + PLAYER_CAPSULE_RADIUS,
            ];

            if world.terrain_resident(min, max) {
                let mut floor_contact = false;
                let mut ceiling_contact = false;

                let output = world.slide_mover(
                    &capsule,
                    start_position,
                    movement_velocity,
                    duration,
                    &MoveAndSlideConfig::default(),
                    |hit| {
                        let normal = **hit.normal;
                        let hit_velocity = *hit.velocity;

                        if normal.y >= GROUND_NORMAL_THRESHOLD
                            && hit_velocity.y <= 0.0
                        {
                            floor_contact = true;

                            if grounded && normal.y > 0.0 {
                                *hit.velocity = Vec3::new(
                                    hit_velocity.x,
                                    -(hit_velocity.x * normal.x
                                        + hit_velocity.z * normal.z)
                                        / normal.y,
                                    hit_velocity.z,
                                );
                            }
                        }

                        if normal.y <= -GROUND_NORMAL_THRESHOLD
                            && hit_velocity.y > 0.0
                        {
                            ceiling_contact = true;
                        }

                        MoveAndSlideHitResponse::Accept
                    },
                );

                transform.translation = output.position;

                if floor_contact {
                    velocity.0.y = 0.0;
                    grounded = true;
                }

                if ceiling_contact {
                    velocity.0.y = 0.0;
                }

                if velocity.0.y <= 0.0 {
                    if let Some(hit) = world.cast_mover(
                        &capsule,
                        transform.translation + Vec3::Y * GROUND_PROBE_LIFT,
                        Vec3::NEG_Y * GROUND_PROBE_DISTANCE,
                        PLAYER_SKIN_WIDTH,
                    ) {
                        if hit.face_normal.y >= GROUND_NORMAL_THRESHOLD {
                            transform.translation.y +=
                                GROUND_PROBE_LIFT - hit.distance;
                            velocity.0.y = 0.0;
                        }
                    }
                }
            }
        }

        if let Some(focus) = view_focus.as_deref_mut() {
            *focus = ViewFocus::body(
                bevy_to_wow(transform.translation),
                world_ready,
            );
        }

        if let Some(viewer) = viewer.as_deref_mut() {
            *viewer = Viewer {
                at: Some(transform.translation),
                planar_speed: horizontal_velocity.length(),
                height: PLAYER_CAPSULE_HEIGHT,
                ..Viewer::default()
            };
        }
    }
}
