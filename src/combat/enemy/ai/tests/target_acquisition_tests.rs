use super::*;

#[test]
fn enemy_prefers_the_player_with_the_highest_threat() {
    let mut app = App::new();
    app.add_systems(Update, enemy_aggro_system);

    let current_target = spawn_living_player(app.world_mut(), Vec3::new(0.0, 0.0, 2.0));

    let highest_threat_player = spawn_living_player(app.world_mut(), Vec3::new(0.0, 0.0, 10.0));

    let enemy = app
        .world_mut()
        .spawn((
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
        ))
        .id();

    app.update();

    assert_eq!(
        app.world().get::<EnemyAi>(enemy).unwrap().target,
        Some(highest_threat_player)
    );
}

#[test]
fn enemy_prefers_ally_summon_with_the_highest_threat() {
    let mut app = setup_enemy_aggro_test_app();

    let player = spawn_living_player(app.world_mut(), Vec3::new(0.0, 0.0, 2.0));

    let ally_summon =
        spawn_living_ally_summon(app.world_mut(), Vec3::new(0.0, 0.0, 10.0), 100.0, 100.0);

    let enemy = app
        .world_mut()
        .spawn((
            Enemy,
            EnemyAi {
                target: Some(player),
                ..EnemyAi::default()
            },
            EnemyHomePosition(Vec3::ZERO),
            EnemyThreat {
                values: std::collections::HashMap::from([(player, 20.0), (ally_summon, 60.0)]),
            },
            CombatStats {
                current_health: 100.0,
                max_health: 100.0,
                resource: 0.0,
            },
            Transform::default(),
        ))
        .id();

    app.update();

    assert_eq!(
        app.world().get::<EnemyAi>(enemy).unwrap().target,
        Some(ally_summon)
    );
}

#[test]
fn enemy_acquires_ally_summon_and_keeps_it_as_target() {
    let mut app = setup_enemy_aggro_test_app();

    let enemy = spawn_enemy_ai(app.world_mut(), Vec3::ZERO);

    let ally_summon =
        spawn_living_ally_summon(app.world_mut(), Vec3::new(0.0, 0.0, 12.0), 100.0, 100.0);

    app.update();

    assert_eq!(
        app.world().get::<EnemyAi>(enemy).unwrap().target,
        Some(ally_summon)
    );

    let closer_player = spawn_living_player(app.world_mut(), Vec3::new(0.0, 0.0, 2.0));

    app.update();

    assert_eq!(
        app.world().get::<EnemyAi>(enemy).unwrap().target,
        Some(ally_summon)
    );

    assert_ne!(
        app.world().get::<EnemyAi>(enemy).unwrap().target,
        Some(closer_player)
    );
}

#[test]
fn enemy_ignores_defeated_ally_summons_during_target_acquisition() {
    let mut app = setup_enemy_aggro_test_app();

    let enemy = spawn_enemy_ai(app.world_mut(), Vec3::ZERO);

    let defeated_ally_summon =
        spawn_living_ally_summon(app.world_mut(), Vec3::new(0.0, 0.0, 2.0), 100.0, 0.0);

    let living_player = spawn_living_player(app.world_mut(), Vec3::new(0.0, 0.0, 8.0));

    app.update();

    assert_eq!(
        app.world().get::<EnemyAi>(enemy).unwrap().target,
        Some(living_player)
    );

    assert_ne!(
        app.world().get::<EnemyAi>(enemy).unwrap().target,
        Some(defeated_ally_summon)
    );
}

#[test]
fn enemy_acquires_the_nearest_living_player_in_range() {
    let mut app = setup_enemy_aggro_test_app();

    let enemy = spawn_enemy_ai(app.world_mut(), Vec3::ZERO);

    let farther_player = spawn_living_player(app.world_mut(), Vec3::new(0.0, 0.0, 15.0));

    let nearest_player = spawn_living_player(app.world_mut(), Vec3::new(0.0, 0.0, 8.0));

    let dead_player = app
        .world_mut()
        .spawn((
            Character,
            Transform::from_xyz(0.0, 0.0, 2.0),
            CombatStats {
                current_health: 0.0,
                max_health: 100.0,
                resource: 0.0,
            },
        ))
        .id();

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

    assert_eq!(app.world().get::<EnemyAi>(enemy).unwrap().target, None);
}

#[test]
fn enemy_keeps_its_current_target_while_that_player_is_alive() {
    let mut app = setup_enemy_aggro_test_app();

    let enemy = spawn_enemy_ai(app.world_mut(), Vec3::ZERO);

    let first_player = spawn_living_player(app.world_mut(), Vec3::new(0.0, 0.0, 12.0));

    app.update();

    assert_eq!(
        app.world().get::<EnemyAi>(enemy).unwrap().target,
        Some(first_player)
    );

    let closer_player = spawn_living_player(app.world_mut(), Vec3::new(0.0, 0.0, 2.0));

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

    let first_player = spawn_living_player(app.world_mut(), Vec3::new(0.0, 0.0, 8.0));

    app.update();

    assert_eq!(
        app.world().get::<EnemyAi>(enemy).unwrap().target,
        Some(first_player)
    );

    app.world_mut()
        .get_mut::<CombatStats>(first_player)
        .unwrap()
        .current_health = 0.0;

    let next_player = spawn_living_player(app.world_mut(), Vec3::new(0.0, 0.0, 12.0));

    app.update();

    assert_eq!(
        app.world().get::<EnemyAi>(enemy).unwrap().target,
        Some(next_player)
    );
}

#[test]
fn enemy_with_only_health_pool_can_acquire_player_with_only_health_pool() {
    let mut app = setup_enemy_aggro_test_app();

    let enemy = app
        .world_mut()
        .spawn((
            Enemy,
            EnemyAi::default(),
            EnemyHomePosition(Vec3::ZERO),
            EnemyThreat::default(),
            Transform::default(),
            HealthPool::new(80.0, 100.0).unwrap(),
        ))
        .id();

    let player =
        spawn_player_with_only_health_pool(app.world_mut(), Vec3::new(0.0, 0.0, 5.0), 60.0, 100.0);

    app.update();

    assert_eq!(
        app.world().get::<EnemyAi>(enemy).unwrap().target,
        Some(player)
    );
}

#[test]
fn enemy_does_not_acquire_player_without_health_components() {
    let mut app = setup_enemy_aggro_test_app();

    let enemy = spawn_enemy_ai(app.world_mut(), Vec3::ZERO);

    let player_without_health = app
        .world_mut()
        .spawn((Character, Transform::from_xyz(0.0, 0.0, 1.0)))
        .id();

    let player_with_health = spawn_living_player(app.world_mut(), Vec3::new(0.0, 0.0, 8.0));

    app.update();

    assert_eq!(
        app.world().get::<EnemyAi>(enemy).unwrap().target,
        Some(player_with_health)
    );

    assert_ne!(
        app.world().get::<EnemyAi>(enemy).unwrap().target,
        Some(player_without_health)
    );
}

#[test]
fn enemy_without_health_components_does_not_acquire_targets() {
    let mut app = setup_enemy_aggro_test_app();

    let enemy = app
        .world_mut()
        .spawn((Enemy, EnemyAi::default(), Transform::default()))
        .id();

    spawn_living_player(app.world_mut(), Vec3::new(0.0, 0.0, 5.0));

    app.update();

    assert_eq!(app.world().get::<EnemyAi>(enemy).unwrap().target, None);
}

#[test]
fn non_finite_aggro_range_uses_the_default_range() {
    let mut app = setup_enemy_aggro_test_app();

    let enemy = app
        .world_mut()
        .spawn((
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
        ))
        .id();

    let player = spawn_living_player(app.world_mut(), Vec3::new(0.0, 0.0, 10.0));

    app.update();

    assert_eq!(
        app.world().get::<EnemyAi>(enemy).unwrap().target,
        Some(player)
    );
}
