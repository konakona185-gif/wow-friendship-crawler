use super::*;

fn setup_player_cast_test_app() -> App {
    let mut app = App::new();
    app.add_systems(Update, player_cast_system);
    app
}

fn spawn_enemy_with_health_pool(app: &mut App, health: f32) -> Entity {
    app.world_mut()
        .spawn((
            Enemy,
            EnemyAi::default(),
            EnemyThreat::default(),
            CombatStats {
                current_health: health,
                max_health: 100.0,
                resource: 0.0,
            },
            HealthPool::new(health, 100.0).unwrap(),
        ))
        .id()
}

fn spawn_player_for_cast(
    app: &mut App,
    enemy: Entity,
    role: FellowshipRole,
    resource: f32,
) -> Entity {
    app.world_mut()
        .spawn((
            Character,
            role,
            CombatStats {
                current_health: 100.0,
                max_health: 100.0,
                resource,
            },
            Target {
                entity: Some(enemy),
            },
            CharacterCommand {
                ability_1: true,
                ..default()
            },
        ))
        .id()
}

fn spawn_player_for_cast_with_health_pool(
    app: &mut App,
    enemy: Entity,
    role: FellowshipRole,
    resource: f32,
    stats_health: f32,
    stats_maximum: f32,
    pool_health: f32,
    pool_maximum: f32,
) -> Entity {
    app.world_mut()
        .spawn((
            Character,
            role,
            CombatStats {
                current_health: stats_health,
                max_health: stats_maximum,
                resource,
            },
            Target {
                entity: Some(enemy),
            },
            CharacterCommand {
                ability_1: true,
                ..default()
            },
            HealthPool::new(pool_health, pool_maximum).unwrap(),
        ))
        .id()
}

mod ability_resolution_tests;
mod eligibility_tests;
