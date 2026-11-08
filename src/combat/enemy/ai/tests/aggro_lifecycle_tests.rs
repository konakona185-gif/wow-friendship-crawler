use super::*;

#[test]
fn leashing_enemy_clears_its_threat_table() {
    let mut app = setup_enemy_aggro_test_app();

    let player = spawn_living_player(
        app.world_mut(),
        Vec3::new(41.0, 0.0, 1.0),
    );

    let enemy = app.world_mut().spawn((
        Enemy,
        EnemyAi {
            target: Some(player),
            ..EnemyAi::default()
        },
        EnemyHomePosition(Vec3::ZERO),
        EnemyThreat {
            values: std::collections::HashMap::from([(player, 45.0)]),
        },
        Transform::from_xyz(41.0, 0.0, 0.0),
        CombatStats {
            current_health: 25.0,
            max_health: 100.0,
            resource: 0.0,
        },
    )).id();

    app.update();

    let ai = app.world().get::<EnemyAi>(enemy).unwrap();
    let threat = app.world().get::<EnemyThreat>(enemy).unwrap();

    assert!(ai.returning_home);
    assert_eq!(ai.target, None);
    assert!(threat.values.is_empty());
}

#[test]
fn enemy_uses_health_pool_to_determine_player_liveness_during_aggro() {
    let mut app = setup_enemy_aggro_test_app();

    let enemy = spawn_enemy_ai(app.world_mut(), Vec3::ZERO);

    let defeated_player = spawn_player_with_health_pool(
        app.world_mut(),
        Vec3::new(0.0, 0.0, 2.0),
        100.0,
        0.0,
    );

    let living_player = spawn_player_with_health_pool(
        app.world_mut(),
        Vec3::new(0.0, 0.0, 6.0),
        0.0,
        35.0,
    );

    let ordinary_player = spawn_living_player(
        app.world_mut(),
        Vec3::new(0.0, 0.0, 10.0),
    );

    app.update();

    assert_eq!(
        app.world().get::<EnemyAi>(enemy).unwrap().target,
        Some(living_player)
    );

    assert_ne!(
        app.world().get::<EnemyAi>(enemy).unwrap().target,
        Some(defeated_player)
    );

    assert_ne!(
        app.world().get::<EnemyAi>(enemy).unwrap().target,
        Some(ordinary_player)
    );
}

#[test]
fn enemy_drops_threat_and_reacquires_when_target_health_pool_reaches_zero() {
    let mut app = setup_enemy_aggro_test_app();

    let defeated_player = spawn_player_with_health_pool(
        app.world_mut(),
        Vec3::new(0.0, 0.0, 8.0),
        100.0,
        0.0,
    );

    let next_player = spawn_player_with_health_pool(
        app.world_mut(),
        Vec3::new(0.0, 0.0, 12.0),
        0.0,
        50.0,
    );

    let enemy = app.world_mut().spawn((
        Enemy,
        EnemyAi {
            target: Some(defeated_player),
            ..EnemyAi::default()
        },
        EnemyHomePosition(Vec3::ZERO),
        EnemyThreat {
            values: std::collections::HashMap::from([
                (defeated_player, 80.0),
            ]),
        },
        Transform::default(),
        CombatStats {
            current_health: 100.0,
            max_health: 100.0,
            resource: 0.0,
        },
    )).id();

    app.update();

    let ai = app.world().get::<EnemyAi>(enemy).unwrap();
    let threat = app.world().get::<EnemyThreat>(enemy).unwrap();

    assert_eq!(ai.target, Some(next_player));
    assert!(!threat.values.contains_key(&defeated_player));
    assert!(threat.values.is_empty());
}

#[test]
fn defeated_enemies_do_not_acquire_or_keep_targets() {
    let mut app = setup_enemy_aggro_test_app();

    let enemy = app.world_mut().spawn((
        Enemy,
        EnemyAi::default(),
        Transform::default(),
        CombatStats {
            current_health: 0.0,
            max_health: 100.0,
            resource: 0.0,
        },
    )).id();

    let player = spawn_living_player(
        app.world_mut(),
        Vec3::new(0.0, 0.0, 2.0),
    );

    app.world_mut()
        .get_mut::<EnemyAi>(enemy)
        .unwrap()
        .target = Some(player);

    app.update();

    assert_eq!(
        app.world().get::<EnemyAi>(enemy).unwrap().target,
        None
    );

    assert!(!app.world().get::<EnemyAi>(enemy).unwrap().returning_home);
}

#[test]
fn enemy_health_pool_overrides_stats_when_determining_enemy_liveness() {
    let mut app = setup_enemy_aggro_test_app();

    let first_player = spawn_living_player(
        app.world_mut(),
        Vec3::new(0.0, 0.0, 2.0),
    );

    let second_player = spawn_living_player(
        app.world_mut(),
        Vec3::new(1000.0, 0.0, 2.0),
    );

    let defeated_enemy = app.world_mut().spawn((
        Enemy,
        EnemyAi {
            target: Some(first_player),
            ..EnemyAi::default()
        },
        EnemyThreat {
            values: std::collections::HashMap::from([
                (first_player, 50.0),
            ]),
        },
        Transform::default(),
        CombatStats {
            current_health: 100.0,
            max_health: 200.0,
            resource: 0.0,
        },
        HealthPool::new(0.0, 150.0).unwrap(),
    )).id();

    let living_enemy = app.world_mut().spawn((
        Enemy,
        EnemyAi::default(),
        Transform::from_xyz(1000.0, 0.0, 0.0),
        CombatStats {
            current_health: 0.0,
            max_health: 75.0,
            resource: 0.0,
        },
        HealthPool::new(40.0, 120.0).unwrap(),
    )).id();

    app.update();

    let defeated_ai = app.world().get::<EnemyAi>(defeated_enemy).unwrap();
    let defeated_stats = app.world().get::<CombatStats>(defeated_enemy).unwrap();
    let defeated_threat = app.world().get::<EnemyThreat>(defeated_enemy).unwrap();

    assert_eq!(defeated_ai.target, None);
    assert!(!defeated_ai.returning_home);
    assert_eq!(defeated_stats.current_health, 0.0);
    assert_eq!(defeated_stats.max_health, 150.0);
    assert!(defeated_threat.values.is_empty());

    let living_ai = app.world().get::<EnemyAi>(living_enemy).unwrap();
    let living_stats = app.world().get::<CombatStats>(living_enemy).unwrap();

    assert_eq!(living_ai.target, Some(second_player));
    assert_eq!(living_stats.current_health, 40.0);
    assert_eq!(living_stats.max_health, 120.0);
}

#[test]
fn enemy_with_only_health_pool_recovers_to_pool_maximum_after_leashing() {
    let mut app = setup_enemy_aggro_test_app();

    let player = spawn_living_player(
        app.world_mut(),
        Vec3::new(41.0, 0.0, 1.0),
    );

    let enemy = app.world_mut().spawn((
        Enemy,
        EnemyAi {
            target: Some(player),
            ..EnemyAi::default()
        },
        EnemyHomePosition(Vec3::ZERO),
        EnemyThreat {
            values: std::collections::HashMap::from([
                (player, 45.0),
            ]),
        },
        Transform::from_xyz(41.0, 0.0, 0.0),
        HealthPool::new(25.0, 75.0).unwrap(),
    )).id();

    app.update();

    let ai = app.world().get::<EnemyAi>(enemy).unwrap();
    let threat = app.world().get::<EnemyThreat>(enemy).unwrap();
    let health_pool = app.world().get::<HealthPool>(enemy).unwrap();

    assert_eq!(ai.target, None);
    assert!(ai.returning_home);
    assert_eq!(health_pool.current(), 75.0);
    assert_eq!(health_pool.maximum(), 75.0);
    assert!(threat.values.is_empty());
}

#[test]
fn enemy_starts_returning_home_and_recovers_health_after_exceeding_leash() {
    let mut app = setup_enemy_aggro_test_app();

    let enemy = app.world_mut().spawn((
        Enemy,
        EnemyAi {
            target: None,
            aggro_range: DEFAULT_ENEMY_AGGRO_RANGE,
            leash_range: DEFAULT_ENEMY_LEASH_RANGE,
            returning_home: false,
        },
        EnemyHomePosition(Vec3::ZERO),
        Transform::from_xyz(41.0, 0.0, 0.0),
        CombatStats {
            current_health: 25.0,
            max_health: 40.0,
            resource: 30.0,
        },
        HealthPool::new(25.0, 100.0).unwrap(),
    )).id();

    let player = spawn_living_player(
        app.world_mut(),
        Vec3::new(41.0, 0.0, 1.0),
    );

    app.world_mut()
        .get_mut::<EnemyAi>(enemy)
        .unwrap()
        .target = Some(player);

    app.update();

    let ai = app.world().get::<EnemyAi>(enemy).unwrap();
    let stats = app.world().get::<CombatStats>(enemy).unwrap();
    let health_pool = app.world().get::<HealthPool>(enemy).unwrap();

    assert_eq!(ai.target, None);
    assert!(ai.returning_home);
    assert_eq!(stats.current_health, 100.0);
    assert_eq!(stats.max_health, 100.0);
    assert_eq!(health_pool.current(), health_pool.maximum());
}

#[test]
fn returning_enemy_does_not_reacquire_players() {
    let mut app = setup_enemy_aggro_test_app();

    let enemy = app.world_mut().spawn((
        Enemy,
        EnemyAi {
            target: None,
            aggro_range: DEFAULT_ENEMY_AGGRO_RANGE,
            leash_range: DEFAULT_ENEMY_LEASH_RANGE,
            returning_home: true,
        },
        EnemyHomePosition(Vec3::ZERO),
        Transform::from_xyz(10.0, 0.0, 0.0),
        CombatStats {
            current_health: 100.0,
            max_health: 100.0,
            resource: 0.0,
        },
    )).id();

    let player = spawn_living_player(
        app.world_mut(),
        Vec3::new(10.0, 0.0, 1.0),
    );

    app.update();

    let ai = app.world().get::<EnemyAi>(enemy).unwrap();

    assert_eq!(ai.target, None);
    assert!(ai.returning_home);

    assert_ne!(ai.target, Some(player));
}

#[test]
fn enemy_can_acquire_targets_again_after_returning_home() {
    let mut app = setup_enemy_aggro_test_app();

    let enemy = app.world_mut().spawn((
        Enemy,
        EnemyAi {
            target: None,
            aggro_range: DEFAULT_ENEMY_AGGRO_RANGE,
            leash_range: DEFAULT_ENEMY_LEASH_RANGE,
            returning_home: true,
        },
        EnemyHomePosition(Vec3::ZERO),
        Transform::from_xyz(0.5, 0.0, 0.0),
        CombatStats {
            current_health: 100.0,
            max_health: 100.0,
            resource: 0.0,
        },
    )).id();

    let player = spawn_living_player(
        app.world_mut(),
        Vec3::new(0.0, 0.0, 5.0),
    );

    app.update();

    let ai = app.world().get::<EnemyAi>(enemy).unwrap();

    assert!(!ai.returning_home);
    assert_eq!(ai.target, Some(player));
}

#[test]
fn enemy_does_not_start_returning_at_the_exact_leash_distance() {
    let mut app = setup_enemy_aggro_test_app();

    let enemy = app.world_mut().spawn((
        Enemy,
        EnemyAi::default(),
        EnemyHomePosition(Vec3::ZERO),
        Transform::from_xyz(DEFAULT_ENEMY_LEASH_RANGE, 0.0, 0.0),
        CombatStats {
            current_health: 100.0,
            max_health: 100.0,
            resource: 0.0,
        },
    )).id();

    app.update();

    let ai = app.world().get::<EnemyAi>(enemy).unwrap();

    assert!(!ai.returning_home);
}
