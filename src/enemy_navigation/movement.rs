
use std::time::Duration;

use avian3d::prelude::{
    Collider,
    MoveAndSlideConfig,
    MoveAndSlideHitResponse,
};
use bevy::prelude::*;
use bevy_landmass::prelude::{
    Agent3dBundle,
    AgentDesiredVelocity3d,
    AgentSettings,
    Archipelago3d,
    ArchipelagoRef3d,
    Velocity3d,
};

use benilla_assets::coords::bevy_to_wow;
use benilla_world::collision::WorldCollision;

use crate::combat::{
    Enemy,
    EnemyAi,
    EnemyHomePosition,
    PendingDisplacement,
};

use super::movement_helpers::*;

#[derive(Component)]
pub(super) struct EnemyNavigationAgent;

#[derive(Component, Default)]
pub(super) struct EnemyVerticalMotion {
    velocity: f32,
}

#[derive(Component, Default)]
pub(super) struct EnemyGroundingState {
    steep_support: bool,
}

pub(super) fn spawn_enemy_navigation_agents(
    mut commands: Commands,
    archipelago_query: Query<Entity, With<Archipelago3d>>,
    enemy_query: Query<
        (Entity, &Transform, Option<&EnemyHomePosition>),
        (With<Enemy>, Without<EnemyNavigationAgent>),
    >,
) {
    let Some(archipelago) = archipelago_query.iter().next() else {
        return;
    };

    for (enemy_entity, transform, home_position) in &enemy_query {
        let initial_position = transform.translation;

        let mut enemy_commands = commands.entity(enemy_entity);

        enemy_commands.insert((
            Agent3dBundle {
                agent: Default::default(),
                settings: AgentSettings {
                    radius: ENEMY_CAPSULE_RADIUS,
                    desired_speed: 5.0,
                    max_speed: 7.0,
                },
                archipelago_ref: ArchipelagoRef3d::new(archipelago),
            },
            EnemyNavigationAgent,
            EnemyVerticalMotion::default(),
            EnemyGroundingState::default(),
        ));

        if home_position.is_none() {
            enemy_commands.insert(EnemyHomePosition(initial_position));
        }
    }
}

pub(super) fn move_enemy_agents(
    time: Res<Time>,
    world: Option<WorldCollision<'_, '_>>,
    mut enemy_query: Query<
        (
            &mut Transform,
            &mut Velocity3d,
            &AgentDesiredVelocity3d,
            &mut EnemyVerticalMotion,
            &mut EnemyGroundingState,
            Option<&mut PendingDisplacement>,
        ),
        (With<Enemy>, With<EnemyNavigationAgent>),
    >,
) {
    let delta = time.delta_secs();

    if delta <= 0.0 {
        return;
    }

    let Some(world) = world else {
        return;
    };

    let duration = Duration::from_secs_f32(delta);
    let capsule = Collider::capsule(
        ENEMY_CAPSULE_RADIUS,
        ENEMY_CAPSULE_HEIGHT - 2.0 * ENEMY_CAPSULE_RADIUS,
    );

    for (
        mut transform,
        mut actual_velocity,
        desired_velocity,
        mut vertical_motion,
        mut grounding_state,
        mut pending_displacement,
    ) in &mut enemy_query
    {
        let mut move_direction = desired_velocity.velocity();
        move_direction.y = 0.0;
        move_direction = move_direction.normalize_or_zero();

        let horizontal_velocity =
            desired_velocity.velocity().with_y(0.0);

        let mut grounded = false;
        let mut steep_support = grounding_state.steep_support;

        if vertical_motion.velocity <= 0.0 {
            let probe_distance = if steep_support {
                ENEMY_STEP_UP_HEIGHT + ENEMY_GROUND_PROBE_DISTANCE
            } else {
                vertical_probe_distance(vertical_motion.velocity)
            };

            if let Some(hit) = world.cast_mover(
                &capsule,
                transform.translation
                    + Vec3::Y * ENEMY_GROUND_PROBE_LIFT,
                Vec3::NEG_Y * probe_distance,
                ENEMY_SKIN_WIDTH,
            ) {
                if hit.face_normal.y >= ENEMY_GROUND_NORMAL_THRESHOLD
                    || steep_support
                {
                    if !steep_support {
                        transform.translation.y +=
                            ENEMY_GROUND_PROBE_LIFT - hit.distance;
                    }

                    vertical_motion.velocity = 0.0;
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

        let movement_velocity = if grounded {
            vertical_motion.velocity = 0.0;
            horizontal_velocity
        } else {
            let (end_velocity, average_velocity) =
                fall_step(vertical_motion.velocity, delta);

            vertical_motion.velocity = end_velocity;
            horizontal_velocity + Vec3::Y * average_velocity
        };

        let step_preview = if grounded {
            move_direction * ENEMY_STEP_UP_ADVANCE
        } else {
            Vec3::ZERO
        };

        let start_position = transform.translation;
        let end_position =
            start_position + movement_velocity * delta + step_preview;

        let start_wow = bevy_to_wow(start_position);
        let end_wow = bevy_to_wow(end_position);

        let min = [
            start_wow[0].min(end_wow[0]) - ENEMY_CAPSULE_RADIUS,
            start_wow[1].min(end_wow[1]) - ENEMY_CAPSULE_RADIUS,
        ];

        let max = [
            start_wow[0].max(end_wow[0]) + ENEMY_CAPSULE_RADIUS,
            start_wow[1].max(end_wow[1]) + ENEMY_CAPSULE_RADIUS,
        ];

        if !world.terrain_resident(min, max) {
            actual_velocity.velocity = Vec3::ZERO;
            vertical_motion.velocity = 0.0;
            grounding_state.steep_support = false;
            continue;
        }

        if grounded {
            let travel = horizontal_velocity.length() * delta;

            let step = if travel > 1.0e-6 {
                try_enemy_step_up(
                    &world,
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
                        enemy_walkable_ride_velocity(normal, hit_velocity)
                    {
                        *hit.velocity = ride;
                        return MoveAndSlideHitResponse::Accept;
                    }

                    if ride_to.is_some_and(|ceiling| hit.position.y < ceiling) {
                        if let Some(up) =
                            enemy_foot_cone_ride(normal, hit_velocity)
                        {
                            *hit.velocity = up;
                            rode = true;
                            return MoveAndSlideHitResponse::Accept;
                        }
                    }

                    if let Some(followed) =
                        enemy_steep_contact_shear(normal, hit_velocity)
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

            let cone_reach = horizontal_distance
                * ENEMY_STEP_SLOPE_RATIO
                + ENEMY_STEP_SNAP_SLACK;

            let snap_reach = cone_reach
                + if steep_support {
                    ENEMY_STEP_UP_HEIGHT
                } else if step.is_some_and(|step| step.ride_to.is_none()) {
                    step.map_or(0.0, |step| step.rise)
                } else {
                    0.0
                };

            let mut support_found = false;

            if let Some(hit) = world.cast_mover(
                &capsule,
                transform.translation
                    + Vec3::Y * ENEMY_GROUND_PROBE_LIFT,
                Vec3::NEG_Y * (snap_reach + ENEMY_GROUND_PROBE_LIFT),
                ENEMY_SKIN_WIDTH,
            ) {
                let drop =
                    (hit.distance - ENEMY_GROUND_PROBE_LIFT).max(0.0);

                let (descent, support_continues) =
                    enemy_descent_step(drop, horizontal_distance);

                transform.translation.y -= descent;
                vertical_motion.velocity = 0.0;
                support_found = true;

                if hit.face_normal.y >= ENEMY_GROUND_NORMAL_THRESHOLD {
                    steep_support = support_continues;
                } else {
                    steep_support = drop > ENEMY_STEP_SNAP_SLACK;
                }
            } else if !rode {
                transform.translation.y -= cone_reach;
            }

            if !support_found && !rode {
                steep_support =
                    step.is_some_and(|step| step.ride_to.is_none());

                let (end_velocity, average_velocity) =
                    fall_step(0.0, delta);

                vertical_motion.velocity = end_velocity;

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
                            enemy_steep_contact_shear(normal, hit_velocity)
                        {
                            *hit.velocity = followed;
                        }

                        MoveAndSlideHitResponse::Accept
                    },
                );

                transform.translation = output.position;

                if vertical_motion.velocity <= 0.0 {
                    if let Some(hit) = world.cast_mover(
                        &capsule,
                        transform.translation
                            + Vec3::Y * ENEMY_GROUND_PROBE_LIFT,
                        Vec3::NEG_Y
                            * (ENEMY_LAND_PROBE_DISTANCE
                                + ENEMY_GROUND_PROBE_LIFT),
                        ENEMY_SKIN_WIDTH,
                    ) {
                        if hit.face_normal.y
                            >= ENEMY_GROUND_NORMAL_THRESHOLD
                        {
                            transform.translation.y +=
                                ENEMY_GROUND_PROBE_LIFT - hit.distance;
                            vertical_motion.velocity = 0.0;
                        }
                    }
                }
            }

            if rode {
                steep_support = true;

                if vertical_motion.velocity > 0.0 {
                    vertical_motion.velocity = 0.0;
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

                    if normal.y <= -ENEMY_GROUND_NORMAL_THRESHOLD
                        && hit_velocity.y > 0.0
                    {
                        ceiling_contact = true;
                    }

                    if let Some(followed) =
                        enemy_steep_contact_shear(normal, hit_velocity)
                    {
                        *hit.velocity = followed;
                    }

                    MoveAndSlideHitResponse::Accept
                },
            );

            transform.translation = output.position;

            if ceiling_contact {
                vertical_motion.velocity = 0.0;
            }

            if vertical_motion.velocity <= 0.0 {
                if let Some(hit) = world.cast_mover(
                    &capsule,
                    transform.translation
                        + Vec3::Y * ENEMY_GROUND_PROBE_LIFT,
                    Vec3::NEG_Y
                        * (ENEMY_LAND_PROBE_DISTANCE
                            + ENEMY_GROUND_PROBE_LIFT),
                    ENEMY_SKIN_WIDTH,
                ) {
                    if hit.face_normal.y
                        >= ENEMY_GROUND_NORMAL_THRESHOLD
                    {
                        transform.translation.y +=
                            ENEMY_GROUND_PROBE_LIFT - hit.distance;
                        vertical_motion.velocity = 0.0;
                    }
                }
            }
        }

        grounding_state.steep_support = steep_support;

        if let Some(mut pending_displacement) = pending_displacement {
            let translation = pending_displacement.pending_translation();

            if translation != Vec3::ZERO {
                let end_position = transform.translation + translation;

                if end_position.is_finite() {
                    let start_wow = bevy_to_wow(transform.translation);
                    let end_wow = bevy_to_wow(end_position);

                    let min = [
                        start_wow[0].min(end_wow[0]) - ENEMY_CAPSULE_RADIUS,
                        start_wow[1].min(end_wow[1]) - ENEMY_CAPSULE_RADIUS,
                    ];

                    let max = [
                        start_wow[0].max(end_wow[0]) + ENEMY_CAPSULE_RADIUS,
                        start_wow[1].max(end_wow[1]) + ENEMY_CAPSULE_RADIUS,
                    ];

                    if world.terrain_resident(min, max) {
                        let displacement_velocity = translation / delta;

                        if displacement_velocity.is_finite() {
                            pending_displacement.take();

                            let output = world.slide_mover(
                                &capsule,
                                transform.translation,
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

        actual_velocity.velocity =
            (transform.translation - start_position) / delta;
    }
}
