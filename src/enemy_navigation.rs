
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
    AgentTarget3d,
    Archipelago3d,
    ArchipelagoRef3d,
    Velocity3d,
};

use benilla_assets::coords::bevy_to_wow;
use benilla_world::collision::WorldCollision;

use crate::combat::{Enemy, EnemyAi};

const ENEMY_CAPSULE_RADIUS: f32 = 1.0 / 3.0;
const ENEMY_CAPSULE_HEIGHT: f32 = 2.027_777_7;
const ENEMY_SKIN_WIDTH: f32 = 0.02;
const ENEMY_GROUND_NORMAL_THRESHOLD: f32 = 0.642_788;
const ENEMY_GROUND_PROBE_LIFT: f32 = 0.02;
const ENEMY_GROUND_PROBE_DISTANCE: f32 = 0.18;
const ENEMY_LAND_PROBE_DISTANCE: f32 = 0.03;
const ENEMY_STEP_UP_HEIGHT: f32 = 1.0;
const ENEMY_STEP_UP_ADVANCE: f32 = 1.191_753_6;
const ENEMY_STEP_SLOPE_RATIO: f32 = 1.849_399;
const ENEMY_STEP_SNAP_SLACK: f32 = 0.027_777_8;
const ENEMY_FOOT_CONE_HEIGHT: f32 =
    ENEMY_CAPSULE_RADIUS * ENEMY_STEP_SLOPE_RATIO;
const ENEMY_OVERHANG_EPS: f32 = -1.0e-4;
const ENEMY_GRAVITY: f32 = -25.0;
const ENEMY_TERMINAL_FALL_SPEED: f32 = 60.148_003;
const ENEMY_MOVE_SPEED: f32 = 5.0;
const ENEMY_MAX_SPEED: f32 = 7.0;

#[derive(Component)]
struct EnemyNavigationAgent;

#[derive(Component, Default)]
struct EnemyVerticalMotion {
    velocity: f32,
}

#[derive(Component, Default)]
struct EnemyGroundingState {
    steep_support: bool,
}

#[derive(Clone, Copy)]
struct EnemyStepUp {
    rise: f32,
    ride_to: Option<f32>,
}

pub struct EnemyNavigationPlugin;

impl Plugin for EnemyNavigationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_enemy_navigation_agents,
                move_enemy_agents,
            )
                .chain(),
        )
        .add_systems(
            PostUpdate,
            sync_enemy_navigation_targets,
        );
    }
}

fn spawn_enemy_navigation_agents(
    mut commands: Commands,
    archipelago_query: Query<Entity, With<Archipelago3d>>,
    enemy_query: Query<
        Entity,
        (With<Enemy>, Without<EnemyNavigationAgent>),
    >,
) {
    let Some(archipelago) = archipelago_query.iter().next() else {
        return;
    };

    for enemy_entity in &enemy_query {
        commands.entity(enemy_entity).insert((
            Agent3dBundle {
                agent: Default::default(),
                settings: AgentSettings {
                    radius: ENEMY_CAPSULE_RADIUS,
                    desired_speed: ENEMY_MOVE_SPEED,
                    max_speed: ENEMY_MAX_SPEED,
                },
                archipelago_ref: ArchipelagoRef3d::new(archipelago),
            },
            EnemyNavigationAgent,
            EnemyVerticalMotion::default(),
            EnemyGroundingState::default(),
        ));
    }
}

fn sync_enemy_navigation_targets(
    enemy_query: Query<
        (&EnemyAi, &mut AgentTarget3d),
        (With<Enemy>, With<EnemyNavigationAgent>),
    >,
) {
    for (enemy_ai, mut navigation_target) in enemy_query {
        match enemy_ai.target {
            Some(target_entity) => {
                let already_targeted = matches!(
                    &*navigation_target,
                    AgentTarget3d::Entity(current)
                        if *current == target_entity
                );

                if !already_targeted {
                    *navigation_target =
                        AgentTarget3d::Entity(target_entity);
                }
            }
            None => {
                if !matches!(&*navigation_target, AgentTarget3d::None) {
                    *navigation_target = AgentTarget3d::None;
                }
            }
        }
    }
}

fn move_enemy_agents(
    time: Res<Time>,
    world: Option<WorldCollision<'_, '_>>,
    mut enemy_query: Query<
        (
            &mut Transform,
            &mut Velocity3d,
            &AgentDesiredVelocity3d,
            &mut EnemyVerticalMotion,
            &mut EnemyGroundingState,
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
    ) in &mut enemy_query {
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

        actual_velocity.velocity =
            (transform.translation - start_position) / delta;
    }
}

fn enemy_descent_step(
    drop: f32,
    horizontal_distance: f32,
) -> (f32, bool) {
    let cone_reach = horizontal_distance
        * ENEMY_STEP_SLOPE_RATIO
        + ENEMY_STEP_SNAP_SLACK;

    (drop.min(cone_reach), drop > cone_reach)
}

fn vertical_probe_distance(vertical_velocity: f32) -> f32 {
    if vertical_velocity < 0.0 {
        ENEMY_LAND_PROBE_DISTANCE
    } else {
        ENEMY_GROUND_PROBE_DISTANCE
    }
}

fn fall_step(initial_velocity: f32, delta: f32) -> (f32, f32) {
    let terminal_velocity = -ENEMY_TERMINAL_FALL_SPEED;
    let unconstrained_end = initial_velocity + ENEMY_GRAVITY * delta;

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
        (terminal_velocity - initial_velocity) / ENEMY_GRAVITY;

    let displacement =
        0.5 * (initial_velocity + terminal_velocity) * time_to_terminal
        + terminal_velocity * (delta - time_to_terminal);

    (terminal_velocity, displacement / delta)
}

fn try_enemy_step_up(
    world: &WorldCollision<'_, '_>,
    capsule: &Collider,
    center: Vec3,
    direction: Vec3,
    travel: f32,
) -> Option<EnemyStepUp> {
    let cast = |from: Vec3, movement: Vec3| {
        world.cast_mover(
            capsule,
            from,
            movement,
            ENEMY_SKIN_WIDTH,
        )
    };

    let ahead = cast(center, direction * travel)?;
    let normal = ahead.face_normal;

    if normal.y >= ENEMY_GROUND_NORMAL_THRESHOLD
        || normal.y < ENEMY_OVERHANG_EPS
        || normal.dot(direction) >= 0.0
    {
        return None;
    }

    let feet_y = center.y - ENEMY_CAPSULE_HEIGHT * 0.5;
    let contact_height = ahead.point1.y - feet_y;

    let rise = cast(
        center,
        Vec3::Y * ENEMY_STEP_UP_HEIGHT,
    )
    .map_or(ENEMY_STEP_UP_HEIGHT, |hit| hit.distance);

    if rise < 1.0e-3 {
        return None;
    }

    let raised = center + Vec3::Y * rise;
    let advance = travel.max(ENEMY_STEP_UP_ADVANCE);

    let forward_distance = cast(
        raised,
        direction * advance,
    )
    .map_or(advance, |hit| hit.distance);

    let over = raised + direction * forward_distance;

    let reach = rise
        + advance * ENEMY_STEP_SLOPE_RATIO
        + ENEMY_STEP_SNAP_SLACK;

    let down = cast(over, Vec3::NEG_Y * reach)?;

    if down.face_normal.y < ENEMY_GROUND_NORMAL_THRESHOLD {
        return None;
    }

    let landed = over + Vec3::NEG_Y * down.distance;
    let height_gain = landed.y - center.y;

    if height_gain <= 0.05 {
        return None;
    }

    let ride_to = (contact_height <= ENEMY_FOOT_CONE_HEIGHT)
        .then_some(landed.y);

    Some(EnemyStepUp {
        rise,
        ride_to,
    })
}

fn enemy_walkable_ride_velocity(
    normal: Vec3,
    velocity: Vec3,
) -> Option<Vec3> {
    if normal.y < ENEMY_GROUND_NORMAL_THRESHOLD
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

fn enemy_foot_cone_ride(
    normal: Vec3,
    velocity: Vec3,
) -> Option<Vec3> {
    if !(ENEMY_OVERHANG_EPS..ENEMY_GROUND_NORMAL_THRESHOLD)
        .contains(&normal.y)
    {
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
        into * ENEMY_STEP_SLOPE_RATIO,
        velocity.z,
    ))
}

fn enemy_steep_contact_shear(
    normal: Vec3,
    velocity: Vec3,
) -> Option<Vec3> {
    if !(ENEMY_OVERHANG_EPS..ENEMY_GROUND_NORMAL_THRESHOLD)
        .contains(&normal.y)
    {
        return None;
    }

    let normal_velocity = velocity.dot(normal);

    if normal_velocity >= 0.0 {
        return None;
    }

    let horizontal_normal = Vec3::new(normal.x, 0.0, normal.z);
    let horizontal_normal_length_squared =
        horizontal_normal.length_squared();

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
        assert_eq!(
            vertical_probe_distance(0.0),
            ENEMY_GROUND_PROBE_DISTANCE
        );
        assert_eq!(
            vertical_probe_distance(1.0),
            ENEMY_GROUND_PROBE_DISTANCE
        );
        assert_eq!(
            vertical_probe_distance(-0.01),
            ENEMY_LAND_PROBE_DISTANCE
        );
        assert!(
            ENEMY_LAND_PROBE_DISTANCE < ENEMY_GROUND_PROBE_DISTANCE
        );
    }

    #[test]
    fn descent_is_limited_to_the_foot_cone_reach() {
        let horizontal_distance = 0.1;
        let cone_reach = horizontal_distance
            * ENEMY_STEP_SLOPE_RATIO
            + ENEMY_STEP_SNAP_SLACK;

        let (descent, support_continues) =
            enemy_descent_step(cone_reach * 2.0, horizontal_distance);

        assert!((descent - cone_reach).abs() < 1.0e-6);
        assert!(support_continues);

        let (full_descent, support_ends) =
            enemy_descent_step(cone_reach * 0.5, horizontal_distance);

        assert!((full_descent - cone_reach * 0.5).abs() < 1.0e-6);
        assert!(!support_ends);
    }

    #[test]
    fn falling_uses_average_velocity_and_respects_terminal_speed() {
        let delta = 1.0 / 60.0;
        let (end_velocity, average_velocity) = fall_step(0.0, delta);

        assert!((end_velocity - ENEMY_GRAVITY * delta).abs() < 1.0e-6);
        assert!(
            (average_velocity - 0.5 * ENEMY_GRAVITY * delta).abs()
                < 1.0e-6
        );

        let (terminal_end, terminal_average) =
            fall_step(-ENEMY_TERMINAL_FALL_SPEED, delta);

        assert_eq!(terminal_end, -ENEMY_TERMINAL_FALL_SPEED);
        assert_eq!(terminal_average, -ENEMY_TERMINAL_FALL_SPEED);
    }
}
