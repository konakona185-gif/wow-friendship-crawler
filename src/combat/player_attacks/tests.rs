
use super::*;

fn setup_player_cast_test_app() -> App {
    let mut app = App::new();
    app.add_systems(Update, player_cast_system);
    app
}

fn spawn_enemy_with_health_pool(
    app: &mut App,
    health: f32,
) -> Entity {
    app.world_mut().spawn((
        Enemy,
        EnemyAi::default(),
        EnemyThreat::default(),
        CombatStats {
            current_health: health,
            max_health: 100.0,
            resource: 0.0,
        },
        HealthPool::new(health, 100.0).unwrap(),
    )).id()
}

fn spawn_player_for_cast(
    app: &mut App,
    enemy: Entity,
    role: FellowshipRole,
    resource: f32,
) -> Entity {
    app.world_mut().spawn((
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
    )).id()
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
    app.world_mut().spawn((
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
    )).id()
}

#[test]
fn defeated_targets_cannot_receive_additional_player_casts() {
    let mut app = App::new();
    app.add_systems(Update, player_cast_system);

    let enemy = app.world_mut().spawn((
        Enemy,
        CombatStats {
            current_health: 0.0,
            max_health: 100.0,
            resource: 0.0,
        },
    )).id();

    let player = app.world_mut().spawn((
        Character,
        FellowshipRole::Dps,
        CombatStats {
            current_health: 100.0,
            max_health: 100.0,
            resource: 100.0,
        },
        Target {
            entity: Some(enemy),
        },
        CharacterCommand {
            ability_1: true,
            ..default()
        },
    )).id();

    app.update();

    assert_eq!(
        app.world().get::<CombatStats>(enemy).unwrap().current_health,
        0.0
    );

    assert_eq!(
        app.world().get::<CombatStats>(player).unwrap().resource,
        100.0
    );
}

#[test]
fn defeated_targets_with_health_pool_cannot_receive_player_casts() {
    let mut app = setup_player_cast_test_app();

    let enemy = app.world_mut().spawn((
        Enemy,
        EnemyAi::default(),
        EnemyThreat::default(),
        CombatStats {
            current_health: 100.0,
            max_health: 100.0,
            resource: 0.0,
        },
        HealthPool::new(0.0, 100.0).unwrap(),
    )).id();

    let player = spawn_player_for_cast(
        &mut app,
        enemy,
        FellowshipRole::Dps,
        100.0,
    );

    app.update();

    let enemy_pool = app.world().get::<HealthPool>(enemy).unwrap();
    let enemy_stats = app.world().get::<CombatStats>(enemy).unwrap();
    let player_stats = app.world().get::<CombatStats>(player).unwrap();

    assert_eq!(enemy_pool.current(), 0.0);
    assert_eq!(enemy_stats.current_health, 0.0);
    assert_eq!(player_stats.resource, 100.0);
}

#[test]
fn living_targets_with_health_pool_can_receive_player_casts_when_stats_are_stale() {
    let mut app = setup_player_cast_test_app();

    let enemy = app.world_mut().spawn((
        Enemy,
        EnemyAi::default(),
        EnemyThreat::default(),
        CombatStats {
            current_health: 0.0,
            max_health: 80.0,
            resource: 0.0,
        },
        HealthPool::new(5.0, 100.0).unwrap(),
    )).id();

    let player = spawn_player_for_cast(
        &mut app,
        enemy,
        FellowshipRole::Tank,
        0.0,
    );

    app.update();

    let enemy_pool = app.world().get::<HealthPool>(enemy).unwrap();
    let enemy_stats = app.world().get::<CombatStats>(enemy).unwrap();
    let player_stats = app.world().get::<CombatStats>(player).unwrap();
    let threat = app.world().get::<EnemyThreat>(enemy).unwrap();

    assert_eq!(enemy_pool.current(), 0.0);
    assert_eq!(enemy_stats.current_health, 0.0);
    assert_eq!(enemy_stats.max_health, 100.0);
    assert_eq!(player_stats.resource, 20.0);

    assert_eq!(
        threat.values.get(&player),
        Some(&(5.0 * DEFAULT_TANK_THREAT_MULTIPLIER))
    );
}

#[test]
fn returning_enemies_are_immune_to_player_abilities() {
    let mut app = App::new();
    app.add_systems(Update, player_cast_system);

    let enemy = app.world_mut().spawn((
        Enemy,
        EnemyAi {
            target: None,
            aggro_range: DEFAULT_ENEMY_AGGRO_RANGE,
            leash_range: DEFAULT_ENEMY_LEASH_RANGE,
            returning_home: true,
        },
        EnemyHomePosition(Vec3::ZERO),
        Transform::from_xyz(41.0, 0.0, 0.0),
        CombatStats {
            current_health: 100.0,
            max_health: 100.0,
            resource: 0.0,
        },
    )).id();

    let player = app.world_mut().spawn((
        Character,
        FellowshipRole::Dps,
        CombatStats {
            current_health: 100.0,
            max_health: 100.0,
            resource: 100.0,
        },
        Target {
            entity: Some(enemy),
        },
        CharacterCommand {
            ability_1: true,
            ..default()
        },
    )).id();

    app.update();

    assert_eq!(
        app.world().get::<CombatStats>(enemy).unwrap().current_health,
        100.0
    );

    assert_eq!(
        app.world().get::<CombatStats>(player).unwrap().resource,
        100.0
    );
}

#[test]
fn defeated_players_cannot_cast_abilities() {
    let mut app = App::new();
    app.add_systems(Update, player_cast_system);

    let enemy = app.world_mut().spawn((
        Enemy,
        CombatStats {
            current_health: 100.0,
            max_health: 100.0,
            resource: 0.0,
        },
    )).id();

    let player = app.world_mut().spawn((
        Character,
        FellowshipRole::Tank,
        CombatStats {
            current_health: 0.0,
            max_health: 100.0,
            resource: 0.0,
        },
        Target {
            entity: Some(enemy),
        },
        CharacterCommand {
            ability_1: true,
            ..default()
        },
    )).id();

    app.update();

    assert_eq!(
        app.world().get::<CombatStats>(enemy).unwrap().current_health,
        100.0
    );

    assert_eq!(
        app.world().get::<CombatStats>(player).unwrap().resource,
        0.0
    );
}

#[test]
fn defeated_players_with_health_pool_cannot_cast_abilities() {
    let mut app = setup_player_cast_test_app();

    let enemy = spawn_enemy_with_health_pool(&mut app, 100.0);

    let player = spawn_player_for_cast_with_health_pool(
        &mut app,
        enemy,
        FellowshipRole::Dps,
        100.0,
        100.0,
        100.0,
        0.0,
        100.0,
    );

    app.update();

    let enemy_pool = app.world().get::<HealthPool>(enemy).unwrap();
    let enemy_stats = app.world().get::<CombatStats>(enemy).unwrap();
    let player_stats = app.world().get::<CombatStats>(player).unwrap();

    assert_eq!(enemy_pool.current(), 100.0);
    assert_eq!(enemy_stats.current_health, 100.0);
    assert_eq!(player_stats.current_health, 0.0);
    assert_eq!(player_stats.max_health, 100.0);
    assert_eq!(player_stats.resource, 100.0);
}

#[test]
fn living_players_with_health_pool_can_cast_when_stats_are_stale() {
    let mut app = setup_player_cast_test_app();

    let enemy = spawn_enemy_with_health_pool(&mut app, 100.0);

    let player = spawn_player_for_cast_with_health_pool(
        &mut app,
        enemy,
        FellowshipRole::Tank,
        0.0,
        0.0,
        80.0,
        100.0,
        120.0,
    );

    app.update();

    let enemy_pool = app.world().get::<HealthPool>(enemy).unwrap();
    let enemy_stats = app.world().get::<CombatStats>(enemy).unwrap();
    let player_stats = app.world().get::<CombatStats>(player).unwrap();

    assert_eq!(enemy_pool.current(), 85.0);
    assert_eq!(enemy_stats.current_health, 85.0);
    assert_eq!(player_stats.current_health, 100.0);
    assert_eq!(player_stats.max_health, 120.0);
    assert_eq!(player_stats.resource, 20.0);
}

#[test]
fn player_damage_never_reduces_enemy_health_below_zero() {
    let mut app = App::new();
    app.add_systems(Update, player_cast_system);

    let enemy = app.world_mut().spawn((
        Enemy,
        CombatStats {
            current_health: 5.0,
            max_health: 100.0,
            resource: 0.0,
        },
    )).id();

    let player = app.world_mut().spawn((
        Character,
        FellowshipRole::Tank,
        CombatStats {
            current_health: 100.0,
            max_health: 100.0,
            resource: 0.0,
        },
        Target {
            entity: Some(enemy),
        },
        CharacterCommand {
            ability_1: true,
            ..default()
        },
    )).id();

    app.update();

    assert_eq!(
        app.world().get::<CombatStats>(enemy).unwrap().current_health,
        0.0
    );

    assert_eq!(
        app.world().get::<CombatStats>(player).unwrap().resource,
        20.0
    );
}

#[test]
fn tank_damage_generates_weighted_threat() {
    let mut app = App::new();
    app.add_systems(Update, player_cast_system);

    let enemy = app.world_mut().spawn((
        Enemy,
        EnemyAi::default(),
        EnemyThreat::default(),
        CombatStats {
            current_health: 100.0,
            max_health: 100.0,
            resource: 0.0,
        },
    )).id();

    let player = app.world_mut().spawn((
        Character,
        FellowshipRole::Tank,
        CombatStats {
            current_health: 100.0,
            max_health: 100.0,
            resource: 0.0,
        },
        Target {
            entity: Some(enemy),
        },
        CharacterCommand {
            ability_1: true,
            ..default()
        },
    )).id();

    app.update();

    let threat = app.world().get::<EnemyThreat>(enemy).unwrap();

    assert_eq!(
        threat.values.get(&player),
        Some(&(15.0 * DEFAULT_TANK_THREAT_MULTIPLIER))
    );
}

#[test]
fn dps_special_uses_health_pool_and_generates_correct_threat() {
    let mut app = setup_player_cast_test_app();

    let enemy = spawn_enemy_with_health_pool(&mut app, 100.0);
    let player = spawn_player_for_cast(
        &mut app,
        enemy,
        FellowshipRole::Dps,
        100.0,
    );

    app.update();

    let health_pool = app.world().get::<HealthPool>(enemy).unwrap();
    let enemy_stats = app.world().get::<CombatStats>(enemy).unwrap();
    let player_stats = app.world().get::<CombatStats>(player).unwrap();
    let threat = app.world().get::<EnemyThreat>(enemy).unwrap();

    assert_eq!(health_pool.current(), 60.0);
    assert_eq!(enemy_stats.current_health, 60.0);
    assert_eq!(player_stats.resource, 70.0);
    assert_eq!(threat.values.get(&player), Some(&40.0));
}

#[test]
fn dps_basic_uses_health_pool_and_generates_correct_threat() {
    let mut app = setup_player_cast_test_app();

    let enemy = spawn_enemy_with_health_pool(&mut app, 100.0);
    let player = spawn_player_for_cast(
        &mut app,
        enemy,
        FellowshipRole::Dps,
        0.0,
    );

    app.update();

    let health_pool = app.world().get::<HealthPool>(enemy).unwrap();
    let enemy_stats = app.world().get::<CombatStats>(enemy).unwrap();
    let player_stats = app.world().get::<CombatStats>(player).unwrap();
    let threat = app.world().get::<EnemyThreat>(enemy).unwrap();

    assert_eq!(health_pool.current(), 90.0);
    assert_eq!(enemy_stats.current_health, 90.0);
    assert_eq!(player_stats.resource, 15.0);
    assert_eq!(threat.values.get(&player), Some(&10.0));
}

#[test]
fn healer_spell_uses_health_pool_and_generates_correct_threat() {
    let mut app = setup_player_cast_test_app();

    let enemy = spawn_enemy_with_health_pool(&mut app, 100.0);
    let player = spawn_player_for_cast(
        &mut app,
        enemy,
        FellowshipRole::Healer,
        20.0,
    );

    app.update();

    let health_pool = app.world().get::<HealthPool>(enemy).unwrap();
    let enemy_stats = app.world().get::<CombatStats>(enemy).unwrap();
    let player_stats = app.world().get::<CombatStats>(player).unwrap();
    let threat = app.world().get::<EnemyThreat>(enemy).unwrap();

    assert_eq!(health_pool.current(), 75.0);
    assert_eq!(enemy_stats.current_health, 75.0);
    assert_eq!(player_stats.resource, 0.0);
    assert_eq!(threat.values.get(&player), Some(&25.0));
}

#[test]
fn tank_health_pool_damage_is_capped_at_remaining_health() {
    let mut app = setup_player_cast_test_app();

    let enemy = spawn_enemy_with_health_pool(&mut app, 5.0);
    let player = spawn_player_for_cast(
        &mut app,
        enemy,
        FellowshipRole::Tank,
        0.0,
    );

    app.update();

    let health_pool = app.world().get::<HealthPool>(enemy).unwrap();
    let enemy_stats = app.world().get::<CombatStats>(enemy).unwrap();
    let player_stats = app.world().get::<CombatStats>(player).unwrap();
    let threat = app.world().get::<EnemyThreat>(enemy).unwrap();

    assert_eq!(health_pool.current(), 0.0);
    assert_eq!(enemy_stats.current_health, 0.0);
    assert_eq!(player_stats.resource, 20.0);

    assert_eq!(
        threat.values.get(&player),
        Some(&(5.0 * DEFAULT_TANK_THREAT_MULTIPLIER))
    );
}

#[test]
fn player_can_attack_enemy_with_only_health_pool() {
    let mut app = setup_player_cast_test_app();

    let enemy = app.world_mut().spawn((
        Enemy,
        EnemyAi::default(),
        EnemyThreat::default(),
        HealthPool::new(100.0, 120.0).unwrap(),
    )).id();

    let player = spawn_player_for_cast(
        &mut app,
        enemy,
        FellowshipRole::Tank,
        0.0,
    );

    app.update();

    let health_pool = app.world().get::<HealthPool>(enemy).unwrap();
    let threat = app.world().get::<EnemyThreat>(enemy).unwrap();
    let player_stats = app.world().get::<CombatStats>(player).unwrap();

    assert_eq!(health_pool.current(), 85.0);
    assert_eq!(health_pool.maximum(), 120.0);
    assert!(app.world().get::<CombatStats>(enemy).is_none());
    assert_eq!(player_stats.resource, 20.0);

    assert_eq!(
        threat.values.get(&player),
        Some(&(15.0 * DEFAULT_TANK_THREAT_MULTIPLIER))
    );
}

#[test]
fn player_cannot_attack_enemy_without_health_components() {
    let mut app = setup_player_cast_test_app();

    let enemy = app.world_mut().spawn((
        Enemy,
        EnemyAi::default(),
        EnemyThreat::default(),
    )).id();

    let player = spawn_player_for_cast(
        &mut app,
        enemy,
        FellowshipRole::Dps,
        0.0,
    );

    app.update();

    let player_stats = app.world().get::<CombatStats>(player).unwrap();
    let threat = app.world().get::<EnemyThreat>(enemy).unwrap();

    assert!(app.world().get::<HealthPool>(enemy).is_none());
    assert!(app.world().get::<CombatStats>(enemy).is_none());
    assert_eq!(player_stats.resource, 0.0);
    assert!(threat.values.get(&player).is_none());
}
