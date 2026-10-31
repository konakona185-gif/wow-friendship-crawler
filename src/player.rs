
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
const GROUND_NORMAL_THRESHOLD: f32 = 0.642_788;
const GROUND_PROBE_LIFT: f32 = 0.02;
const GROUND_PROBE_DISTANCE: f32 = 0.18;
const LAND_PROBE_DISTANCE: f32 = 0.03;
const STEP_UP_HEIGHT: f32 = 1.0;
const STEP_UP_ADVANCE: f32 = 1.191_753_6;
const STEP_SLOPE_RATIO: f32 = 1.849_399;
const STEP_SNAP_SLACK: f32 = 0.027_777_8;
const FOOT_CONE_HEIGHT: f32 = PLAYER_CAPSULE_RADIUS * STEP_SLOPE_RATIO;
const OVERHANG_EPS: f32 = -1.0e-4;
const GRAVITY: f32 = -25.0;
const JUMP_SPEED: f32 = 9.0;
const TERMINAL_FALL_SPEED: f32 = 60.148_003;

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

#[derive(Component, Default)]
pub struct GroundingState {
    pub steep_support: bool,
}

#[derive(Clone, Copy)]
struct StepUp {
    rise: f32,
    ride_to: Option<f32>,
}

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
            Option<&mut GroundingState>,
        ),
        With<Character>,
    >,
) {
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

    for (mut transform, speed, mut velocity, mut command, grounding_state) in &mut query {
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
            let mut steep_support = grounding_state
                .as_deref()
                .is_some_and(|state| state.steep_support);

            if velocity.0.y <= 0.0 {
                let probe_distance = if steep_support {
                    STEP_UP_HEIGHT + GROUND_PROBE_DISTANCE
                } else {
                    vertical_probe_distance(velocity.0.y)
                };

                if let Some(hit) = world.cast_mover(
                    &capsule,
                    transform.translation + Vec3::Y * GROUND_PROBE_LIFT,
                    Vec3::NEG_Y * probe_distance,
                    PLAYER_SKIN_WIDTH,
                ) {
                    if hit.face_normal.y >= GROUND_NORMAL_THRESHOLD || steep_support {
                        if !steep_support {
                            transform.translation.y +=
                                GROUND_PROBE_LIFT - hit.distance;
                        }
                        velocity.0.y = 0.0;
                        grounded = true;
                    } else {
                        steep_support = false;
                    }
                } else {
                    steep_support = false;
                }
            } else {
                steep_support = false;
            }

            if jump_requested && grounded {
                velocity.0.y = JUMP_SPEED;
                grounded = false;
                steep_support = false;
            }

            let grounded_walk = grounded;
            let vertical_move_velocity = if grounded_walk {
                velocity.0.y = 0.0;
                0.0
            } else {
                let (end_velocity, average_velocity) =
                    fall_step(velocity.0.y, delta);
                velocity.0.y = end_velocity;
                average_velocity
            };

            let movement_velocity = if grounded_walk {
                horizontal_velocity
            } else {
                horizontal_velocity + Vec3::Y * vertical_move_velocity
            };

            let step_preview = if grounded_walk {
                move_direction * STEP_UP_ADVANCE
            } else {
                Vec3::ZERO
            };

            let start_position = transform.translation;
            let end_position =
                start_position + movement_velocity * delta + step_preview;

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
                if grounded_walk {
                    let travel = horizontal_velocity.length() * delta;

                    let step = if travel > 1.0e-6 {
                        try_step_up(
                            world,
                            &capsule,
                            start_position,
                            move_direction,
                            travel,
                        )
                    } else {
                        None
                    };

                    let mut slide_start = start_position;

                    if let Some(step) = step {
                        if step.ride_to.is_none() {
                            slide_start.y += step.rise;
                        }
                    }

                    let ride_to = step.and_then(|step| step.ride_to);
                    let mut rode = false;

                    let output = world.slide_mover(
                        &capsule,
                        slide_start,
                        horizontal_velocity,
                        duration,
                        &MoveAndSlideConfig::default(),
                        |hit| {
                            let normal = **hit.normal;
                            let hit_velocity = *hit.velocity;

                            if let Some(ride) =
                                walkable_ride_velocity(normal, hit_velocity)
                            {
                                *hit.velocity = ride;
                                return MoveAndSlideHitResponse::Accept;
                            }

                            if ride_to.is_some_and(|ceiling| hit.position.y < ceiling) {
                                if let Some(up) = foot_cone_ride(
                                    normal,
                                    hit_velocity,
                                ) {
                                    *hit.velocity = up;
                                    rode = true;
                                    return MoveAndSlideHitResponse::Accept;
                                }
                            }

                            if let Some(followed) =
                                steep_contact_shear(normal, hit_velocity)
                            {
                                *hit.velocity = followed;
                            }

                            MoveAndSlideHitResponse::Accept
                        },
                    );

                    transform.translation = output.position;

                    let slide_delta = output.position - slide_start;
                    let horizontal_distance =
                        slide_delta.x.hypot(slide_delta.z);

                    let cone_reach =
                        horizontal_distance * STEP_SLOPE_RATIO + STEP_SNAP_SLACK;
                    let snap_reach = cone_reach
                        + if steep_support {
                            STEP_UP_HEIGHT
                        } else if step.is_some_and(|step| step.ride_to.is_none()) {
                            step.map_or(0.0, |step| step.rise)
                        } else {
                            0.0
                        };

                    let mut support_found = false;

                    if let Some(hit) = world.cast_mover(
                        &capsule,
                        transform.translation + Vec3::Y * GROUND_PROBE_LIFT,
                        Vec3::NEG_Y * (snap_reach + GROUND_PROBE_LIFT),
                        PLAYER_SKIN_WIDTH,
                    ) {
                        let drop = (hit.distance - GROUND_PROBE_LIFT).max(0.0);
                        let (descent, support_continues) =
                            descent_step(drop, horizontal_distance);
                        transform.translation.y -= descent;
                        velocity.0.y = 0.0;
                        support_found = true;

                        if hit.face_normal.y >= GROUND_NORMAL_THRESHOLD {
                            steep_support = support_continues;
                        } else {
                            steep_support = drop > STEP_SNAP_SLACK;
                        }
                    } else if !rode {
                        transform.translation.y -= cone_reach;
                    }

                    if !support_found && !rode {
                        steep_support = step.is_some_and(|step| step.ride_to.is_none());
                        let (end_velocity, average_velocity) = fall_step(0.0, delta);
                        velocity.0.y = end_velocity;

                        let output = world.slide_mover(
                            &capsule,
                            transform.translation,
                            Vec3::Y * average_velocity,
                            duration,
                            &MoveAndSlideConfig::default(),
                            |hit| {
                                let normal = **hit.normal;
                                let hit_velocity = *hit.velocity;

                                if let Some(followed) =
                                    steep_contact_shear(normal, hit_velocity)
                                {
                                    *hit.velocity = followed;
                                }

                                MoveAndSlideHitResponse::Accept
                            },
                        );

                        transform.translation = output.position;

                        if velocity.0.y <= 0.0 {
                            if let Some(hit) = world.cast_mover(
                                &capsule,
                                transform.translation + Vec3::Y * GROUND_PROBE_LIFT,
                                Vec3::NEG_Y * (LAND_PROBE_DISTANCE + GROUND_PROBE_LIFT),
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

                    if rode {
                        steep_support = true;
                        if velocity.0.y > 0.0 {
                            velocity.0.y = 0.0;
                        }
                    }
                } else {
                    steep_support = false;
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

                            if normal.y <= -GROUND_NORMAL_THRESHOLD
                                && hit_velocity.y > 0.0
                            {
                                ceiling_contact = true;
                            }

                            if let Some(followed) =
                                steep_contact_shear(normal, hit_velocity)
                            {
                                *hit.velocity = followed;
                            }

                            MoveAndSlideHitResponse::Accept
                        },
                    );

                    transform.translation = output.position;

                    if ceiling_contact {
                        velocity.0.y = 0.0;
                    }

                    if velocity.0.y <= 0.0 {
                        if let Some(hit) = world.cast_mover(
                            &capsule,
                            transform.translation + Vec3::Y * GROUND_PROBE_LIFT,
                            Vec3::NEG_Y * (LAND_PROBE_DISTANCE + GROUND_PROBE_LIFT),
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

            if let Some(mut state) = grounding_state {
                state.steep_support = steep_support;
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

fn descent_step(drop: f32, horizontal_distance: f32) -> (f32, bool) {
    let cone_reach = horizontal_distance * STEP_SLOPE_RATIO + STEP_SNAP_SLACK;
    (drop.min(cone_reach), drop > cone_reach)
}

fn vertical_probe_distance(vertical_velocity: f32) -> f32 {
    if vertical_velocity < 0.0 {
        LAND_PROBE_DISTANCE
    } else {
        GROUND_PROBE_DISTANCE
    }
}

fn fall_step(initial_velocity: f32, delta: f32) -> (f32, f32) {
    let terminal_velocity = -TERMINAL_FALL_SPEED;
    let unconstrained_end = initial_velocity + GRAVITY * delta;

    if unconstrained_end >= terminal_velocity {
        return (
            unconstrained_end,
            0.5 * (initial_velocity + unconstrained_end),
        );
    }

    if initial_velocity <= terminal_velocity {
        return (terminal_velocity, terminal_velocity);
    }

    let time_to_terminal =
        (terminal_velocity - initial_velocity) / GRAVITY;
    let displacement =
        0.5 * (initial_velocity + terminal_velocity) * time_to_terminal
        + terminal_velocity * (delta - time_to_terminal);

    (terminal_velocity, displacement / delta)
}

fn try_step_up(
    world: &WorldCollision<'_, '_>,
    capsule: &Collider,
    center: Vec3,
    direction: Vec3,
    travel: f32,
) -> Option<StepUp> {
    let cast = |from: Vec3, movement: Vec3| {
        world.cast_mover(
            capsule,
            from,
            movement,
            PLAYER_SKIN_WIDTH,
        )
    };

    let ahead = cast(center, direction * travel)?;
    let normal = ahead.face_normal;

    if normal.y >= GROUND_NORMAL_THRESHOLD
        || normal.y < OVERHANG_EPS
        || normal.dot(direction) >= 0.0
    {
        return None;
    }

    let feet_y = center.y - PLAYER_CAPSULE_HEIGHT * 0.5;
    let contact_height = ahead.point1.y - feet_y;

    let rise = cast(
        center,
        Vec3::Y * STEP_UP_HEIGHT,
    )
    .map_or(STEP_UP_HEIGHT, |hit| hit.distance);

    if rise < 1.0e-3 {
        return None;
    }

    let raised = center + Vec3::Y * rise;
    let advance = travel.max(STEP_UP_ADVANCE);
    let forward_distance = cast(
        raised,
        direction * advance,
    )
    .map_or(advance, |hit| hit.distance);

    let over = raised + direction * forward_distance;
    let reach = rise + advance * STEP_SLOPE_RATIO + STEP_SNAP_SLACK;

    let down = cast(over, Vec3::NEG_Y * reach)?;

    if down.face_normal.y < GROUND_NORMAL_THRESHOLD {
        return None;
    }

    let landed = over + Vec3::NEG_Y * down.distance;
    let height_gain = landed.y - center.y;

    if height_gain <= 0.05 {
        return None;
    }

    let ride_to = (contact_height <= FOOT_CONE_HEIGHT)
        .then_some(landed.y);

    Some(StepUp {
        rise,
        ride_to,
    })
}

fn walkable_ride_velocity(
    normal: Vec3,
    velocity: Vec3,
) -> Option<Vec3> {
    if normal.y < GROUND_NORMAL_THRESHOLD
        || velocity.dot(normal) >= 0.0
    {
        return None;
    }

    Some(Vec3::new(
        velocity.x,
        -(velocity.x * normal.x + velocity.z * normal.z) / normal.y,
        velocity.z,
    ))
}

fn foot_cone_ride(
    normal: Vec3,
    velocity: Vec3,
) -> Option<Vec3> {
    if !(OVERHANG_EPS..GROUND_NORMAL_THRESHOLD).contains(&normal.y) {
        return None;
    }

    let horizontal_normal =
        Vec3::new(normal.x, 0.0, normal.z).normalize_or_zero();

    let into = -(velocity.x * horizontal_normal.x
        + velocity.z * horizontal_normal.z);

    if into <= 0.0 {
        return None;
    }

    Some(Vec3::new(
        velocity.x,
        into * STEP_SLOPE_RATIO,
        velocity.z,
    ))
}

fn steep_contact_shear(
    normal: Vec3,
    velocity: Vec3,
) -> Option<Vec3> {
    if !(OVERHANG_EPS..GROUND_NORMAL_THRESHOLD).contains(&normal.y) {
        return None;
    }

    let normal_velocity = velocity.dot(normal);

    if normal_velocity >= 0.0 {
        return None;
    }

    let horizontal_normal = Vec3::new(normal.x, 0.0, normal.z);
    let horizontal_normal_length_squared = horizontal_normal.length_squared();

    if horizontal_normal_length_squared <= f32::EPSILON {
        return None;
    }

    Some(
        velocity
            - (normal_velocity / horizontal_normal_length_squared)
                * horizontal_normal,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn falling_uses_a_shorter_ground_probe_than_standing() {
        assert_eq!(vertical_probe_distance(0.0), GROUND_PROBE_DISTANCE);
        assert_eq!(vertical_probe_distance(1.0), GROUND_PROBE_DISTANCE);
        assert_eq!(vertical_probe_distance(-0.01), LAND_PROBE_DISTANCE);
        assert!(LAND_PROBE_DISTANCE < GROUND_PROBE_DISTANCE);
    }

    #[test]
    fn descent_is_limited_to_the_foot_cone_reach() {
        let horizontal_distance = 0.1;
        let cone_reach = horizontal_distance * STEP_SLOPE_RATIO + STEP_SNAP_SLACK;
        let (descent, support_continues) =
            descent_step(cone_reach * 2.0, horizontal_distance);

        assert!((descent - cone_reach).abs() < 1.0e-6);
        assert!(support_continues);

        let (full_descent, support_ends) =
            descent_step(cone_reach * 0.5, horizontal_distance);

        assert!((full_descent - cone_reach * 0.5).abs() < 1.0e-6);
        assert!(!support_ends);
    }

    #[test]
    fn falling_uses_average_velocity_and_respects_terminal_speed() {
        let delta = 1.0 / 60.0;
        let (end_velocity, average_velocity) = fall_step(0.0, delta);

        assert!((end_velocity - GRAVITY * delta).abs() < 1.0e-6);
        assert!((average_velocity - 0.5 * GRAVITY * delta).abs() < 1.0e-6);

        let (terminal_end, terminal_average) =
            fall_step(-TERMINAL_FALL_SPEED, delta);

        assert_eq!(terminal_end, -TERMINAL_FALL_SPEED);
        assert_eq!(terminal_average, -TERMINAL_FALL_SPEED);
    }
}
