use super::*;

fn spawn_living_player(
    world: &mut World,
    position: Vec3,
) -> Entity {
    world.spawn((
        Character,
        Transform::from_translation(position),
        CombatStats {
            current_health: 100.0,
            max_health: 100.0,
            resource: 0.0,
        },
    )).id()
}

fn spawn_player_with_health_pool(
    world: &mut World,
    position: Vec3,
    stats_health: f32,
    pool_health: f32,
) -> Entity {
    world.spawn((
        Character,
        Transform::from_translation(position),
        CombatStats {
            current_health: stats_health,
            max_health: 100.0,
            resource: 0.0,
        },
        HealthPool::new(pool_health, 100.0).unwrap(),
    )).id()
}

fn spawn_player_with_only_health_pool(
    world: &mut World,
    position: Vec3,
    health: f32,
    maximum: f32,
) -> Entity {
    world.spawn((
        Character,
        Transform::from_translation(position),
        HealthPool::new(health, maximum).unwrap(),
    )).id()
}

fn spawn_living_ally_summon(
    world: &mut World,
    position: Vec3,
    stats_health: f32,
    pool_health: f32,
) -> Entity {
    world.spawn((
        AllySummon,
        Transform::from_translation(position),
        CombatStats {
            current_health: stats_health,
            max_health: 100.0,
            resource: 0.0,
        },
        HealthPool::new(pool_health, 100.0).unwrap(),
    )).id()
}

fn spawn_enemy_ai(
    world: &mut World,
    position: Vec3,
) -> Entity {
    world.spawn((
        Enemy,
        EnemyAi::default(),
        EnemyHomePosition(position),
        EnemyThreat::default(),
        Transform::from_translation(position),
        CombatStats {
            current_health: 100.0,
            max_health: 100.0,
            resource: 0.0,
        },
    )).id()
}

fn setup_enemy_aggro_test_app() -> App {
    let mut app = App::new();
    app.add_systems(Update, enemy_aggro_system);
    app
}

mod target_acquisition_tests;
mod aggro_lifecycle_tests;
