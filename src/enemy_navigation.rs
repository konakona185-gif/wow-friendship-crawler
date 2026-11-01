
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
    ) in &mut enemy_query {
        let start_position = transform.translation;
        let mut grounded = false;

        if vertical_motion.velocity <= 0.0 {
            if let Some(hit) = world.cast_mover(
                &capsule,
                start_position + Vec3::Y * ENEMY_GROUND_PROBE_LIFT,
                Vec3::NEG_Y * ENEMY_GROUND_PROBE_DISTANCE,
                ENEMY_SKIN_WIDTH,
            ) {
                if hit.face_normal.y >= ENEMY_GROUND_NORMAL_THRESHOLD {
                    transform.translation.y +=
                        ENEMY_GROUND_PROBE_LIFT - hit.distance;
                    vertical_motion.velocity = 0.0;
                    grounded = true;
                }
            }
        }

        let mut movement_velocity = desired_velocity.velocity();

        if grounded {
            vertical_motion.velocity = 0.0;
        } else {
            let (end_velocity, average_velocity) =
                fall_step(vertical_motion.velocity, delta);

            vertical_motion.velocity = end_velocity;
            movement_velocity.y += average_velocity;
        }

        let start_position = transform.translation;
        let end_position =
            start_position + movement_velocity * delta;

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
            continue;
        }

        if movement_velocity.length_squared() <= 1.0e-6 {
            actual_velocity.velocity = Vec3::ZERO;
            continue;
        }

        let output = world.slide_mover(
            &capsule,
            start_position,
            movement_velocity,
            duration,
            &MoveAndSlideConfig::default(),
            |_| MoveAndSlideHitResponse::Accept,
        );

        transform.translation = output.position;

        if vertical_motion.velocity <= 0.0 {
            if let Some(hit) = world.cast_mover(
                &capsule,
                transform.translation
                    + Vec3::Y * ENEMY_GROUND_PROBE_LIFT,
                Vec3::NEG_Y * ENEMY_GROUND_PROBE_DISTANCE,
                ENEMY_SKIN_WIDTH,
            ) {
                if hit.face_normal.y >= ENEMY_GROUND_NORMAL_THRESHOLD {
                    transform.translation.y +=
                        ENEMY_GROUND_PROBE_LIFT - hit.distance;
                    vertical_motion.velocity = 0.0;
                }
            }
        }

        actual_velocity.velocity =
            (transform.translation - start_position) / delta;
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
