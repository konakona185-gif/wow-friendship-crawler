
use std::time::Duration;

use avian3d::prelude::{
    Collider,
    MoveAndSlideConfig,
    MoveAndSlideHitResponse,
};
use bevy::prelude::*;

use benilla_assets::coords::bevy_to_wow;
use benilla_assets::WorldAssets;
use benilla_world::collision::WorldCollision;
use benilla_world::terrain_stream::{
    ViewFocus,
    WorldLoadProgress,
};
use benilla_world::view::Viewer;

use super::{
    Character,
    CharacterCommand,
    GroundingState,
    MovementSpeed,
    PendingDisplacement,
    Velocity,
};
use super::movement_helpers::{
    descent_step,
    fall_step,
    foot_cone_ride,
    steep_contact_shear,
    try_step_up,
    vertical_probe_distance,
    walkable_ride_velocity,
    GROUND_NORMAL_THRESHOLD,
    GROUND_PROBE_DISTANCE,
    GROUND_PROBE_LIFT,
    JUMP_SPEED,
    LAND_PROBE_DISTANCE,
    PLAYER_CAPSULE_HEIGHT,
    PLAYER_CAPSULE_RADIUS,
    PLAYER_SKIN_WIDTH,
    STEP_SLOPE_RATIO,
    STEP_SNAP_SLACK,
    STEP_UP_ADVANCE,
    STEP_UP_HEIGHT,
};

pub(super) fn player_movement_system(
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
            Option<&mut PendingDisplacement>,
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

    for (
        mut transform,
        speed,
        mut velocity,
        mut command,
        grounding_state,
        pending_displacement,
    ) in &mut query
    {
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

        if let (
            Some(world),
            Some(mut pending_displacement),
        ) = (
            world.as_ref().filter(|_| world_ready),
            pending_displacement,
        ) {
            let translation = pending_displacement.pending_translation();

            if translation != Vec3::ZERO {
                let start_position = transform.translation;
                let end_position = start_position + translation;

                if end_position.is_finite() {
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
                        let displacement_velocity = translation / delta;

                        if displacement_velocity.is_finite() {
                            pending_displacement.take();

                            let output = world.slide_mover(
                                &capsule,
                                start_position,
                                displacement_velocity,
                                duration,
                                &MoveAndSlideConfig::default(),
                                |_| MoveAndSlideHitResponse::Accept,
                            );

                            transform.translation = output.position;
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
