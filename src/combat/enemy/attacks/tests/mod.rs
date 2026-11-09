use super::*;
use std::time::Duration;

use crate::player::Character;

fn setup_enemy_attack_test_app() -> App {
    let mut app = App::new();

    app.insert_resource(Time::<()>::default());
    app.add_systems(Update, enemy_attack_system);

    app
}

fn advance_test_time(app: &mut App, seconds: f32) {
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(seconds));
}

fn test_enemy_attack_state(target: Entity) -> EnemyAttackState {
    EnemyAttackState {
        target,
        cooldown: Timer::from_seconds(DEFAULT_ENEMY_ATTACK_INTERVAL, TimerMode::Repeating),
    }
}

fn test_summon_combat_profile(
    attack_damage: f32,
    attack_interval_seconds: f32,
    melee_range: f32,
) -> SummonCombatProfile {
    SummonCombatProfile {
        attack_damage,
        attack_interval_seconds,
        melee_range,
        aggro_range: 15.0,
        leash_range: 40.0,
        movement_speed: 5.0,
    }
}

fn spawn_enemy_summon(world: &mut World, target: Entity, profile: SummonCombatProfile) -> Entity {
    world
        .spawn((
            Enemy,
            profile,
            EnemyAi {
                target: Some(target),
                ..EnemyAi::default()
            },
            Transform::default(),
        ))
        .id()
}

fn spawn_living_player(world: &mut World, position: Vec3) -> Entity {
    world
        .spawn((
            Character,
            Transform::from_translation(position),
            CombatStats {
                current_health: 100.0,
                max_health: 100.0,
                resource: 0.0,
            },
        ))
        .id()
}

fn spawn_player_with_health_pool(
    world: &mut World,
    position: Vec3,
    stats_health: f32,
    stats_maximum: f32,
    pool_health: f32,
    pool_maximum: f32,
) -> Entity {
    world
        .spawn((
            Character,
            Transform::from_translation(position),
            CombatStats {
                current_health: stats_health,
                max_health: stats_maximum,
                resource: 0.0,
            },
            HealthPool::new(pool_health, pool_maximum).unwrap(),
        ))
        .id()
}

fn spawn_living_ally_summon(
    world: &mut World,
    position: Vec3,
    stats_health: f32,
    stats_maximum: f32,
    pool_health: f32,
    pool_maximum: f32,
) -> Entity {
    world
        .spawn((
            AllySummon,
            Transform::from_translation(position),
            CombatStats {
                current_health: stats_health,
                max_health: stats_maximum,
                resource: 0.0,
            },
            HealthPool::new(pool_health, pool_maximum).unwrap(),
        ))
        .id()
}

mod damage_resolution_tests;
mod eligibility_tests;
