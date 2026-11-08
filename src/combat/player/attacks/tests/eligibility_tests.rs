use super::*;

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
