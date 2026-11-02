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

#[test]
fn enemy_prefers_the_player_with_the_highest_threat() {
    let mut app = App::new();
    app.add_systems(Update, enemy_aggro_system);

    let current_target = spawn_living_player(
        app.world_mut(),
        Vec3::new(0.0, 0.0, 2.0),
    );

    let highest_threat_player = spawn_living_player(
        app.world_mut(),
        Vec3::new(0.0, 0.0, 10.0),
    );

    let enemy = app.world_mut().spawn((
        Enemy,
        EnemyAi {
            target: Some(current_target),
            ..EnemyAi::default()
        },
        EnemyHomePosition(Vec3::ZERO),
        EnemyThreat {
            values: std::collections::HashMap::from([
                (current_target, 20.0),
                (highest_threat_player, 60.0),
            ]),
        },
        CombatStats {
            current_health: 100.0,
            max_health: 100.0,
            resource: 0.0,
        },
        Transform::default(),
    )).id();

    app.update();

    assert_eq!(
        app.world().get::<EnemyAi>(enemy).unwrap().target,
        Some(highest_threat_player)
    );
}

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
fn enemy_acquires_the_nearest_living_player_in_range() {
    let mut app = setup_enemy_aggro_test_app();

    let enemy = spawn_enemy_ai(app.world_mut(), Vec3::ZERO);

    let farther_player = spawn_living_player(
        app.world_mut(),
        Vec3::new(0.0, 0.0, 15.0),
    );

    let nearest_player = spawn_living_player(
        app.world_mut(),
        Vec3::new(0.0, 0.0, 8.0),
    );

    let dead_player = app.world_mut().spawn((
        Character,
        Transform::from_xyz(0.0, 0.0, 2.0),
        CombatStats {
            current_health: 0.0,
            max_health: 100.0,
            resource: 0.0,
        },
    )).id();

    app.update();

    assert_eq!(
        app.world().get::<EnemyAi>(enemy).unwrap().target,
        Some(nearest_player)
    );

    assert_ne!(
        app.world().get::<EnemyAi>(enemy).unwrap().target,
        Some(farther_player)
    );

    assert_ne!(
        app.world().get::<EnemyAi>(enemy).unwrap().target,
        Some(dead_player)
    );
}

#[test]
fn enemy_does_not_acquire_a_player_outside_aggro_range() {
    let mut app = setup_enemy_aggro_test_app();

    let enemy = spawn_enemy_ai(app.world_mut(), Vec3::ZERO);

    spawn_living_player(
        app.world_mut(),
        Vec3::new(0.0, 0.0, DEFAULT_ENEMY_AGGRO_RANGE + 1.0),
    );

    app.update();

    assert_eq!(
        app.world().get::<EnemyAi>(enemy).unwrap().target,
        None
    );
}

#[test]
fn enemy_keeps_its_current_target_while_that_player_is_alive() {
    let mut app = setup_enemy_aggro_test_app();

    let enemy = spawn_enemy_ai(app.world_mut(), Vec3::ZERO);

    let first_player = spawn_living_player(
        app.world_mut(),
        Vec3::new(0.0, 0.0, 12.0),
    );

    app.update();

    assert_eq!(
        app.world().get::<EnemyAi>(enemy).unwrap().target,
        Some(first_player)
    );

    let closer_player = spawn_living_player(
        app.world_mut(),
        Vec3::new(0.0, 0.0, 2.0),
    );

    app.update();

    assert_eq!(
        app.world().get::<EnemyAi>(enemy).unwrap().target,
        Some(first_player)
    );

    assert_ne!(
        app.world().get::<EnemyAi>(enemy).unwrap().target,
        Some(closer_player)
    );
}

#[test]
fn enemy_reacquires_a_new_player_when_its_target_dies() {
    let mut app = setup_enemy_aggro_test_app();

    let enemy = spawn_enemy_ai(app.world_mut(), Vec3::ZERO);

    let first_player = spawn_living_player(
        app.world_mut(),
        Vec3::new(0.0, 0.0, 8.0),
    );

    app.update();

    assert_eq!(
        app.world().get::<EnemyAi>(enemy).unwrap().target,
        Some(first_player)
    );

    app.world_mut()
        .get_mut::<CombatStats>(first_player)
        .unwrap()
        .current_health = 0.0;

    let next_player = spawn_living_player(
        app.world_mut(),
        Vec3::new(0.0, 0.0, 12.0),
    );

    app.update();

    assert_eq!(
        app.world().get::<EnemyAi>(enemy).unwrap().target,
        Some(next_player)
    );
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
fn non_finite_aggro_range_uses_the_default_range() {
    let mut app = setup_enemy_aggro_test_app();

    let enemy = app.world_mut().spawn((
        Enemy,
        EnemyAi {
            target: None,
            aggro_range: f32::NAN,
            leash_range: DEFAULT_ENEMY_LEASH_RANGE,
            returning_home: false,
        },
        Transform::default(),
        CombatStats {
            current_health: 100.0,
            max_health: 100.0,
            resource: 0.0,
        },
    )).id();

    let player = spawn_living_player(
        app.world_mut(),
        Vec3::new(0.0, 0.0, 10.0),
    );

    app.update();

    assert_eq!(
        app.world().get::<EnemyAi>(enemy).unwrap().target,
        Some(player)
    );
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
            max_health: 100.0,
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
    assert_eq!(stats.current_health, stats.max_health);
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
