use super::*;

#[test]
fn enemy_summon_uses_its_custom_attack_damage_and_interval() {
    let mut app = setup_enemy_attack_test_app();

    let player = spawn_living_player(
        app.world_mut(),
        Vec3::new(0.0, 0.0, 1.0),
    );

    spawn_enemy_summon(
        app.world_mut(),
        player,
        test_summon_combat_profile(
            17.0,
            1.5,
            4.0,
        ),
    );

    app.update();

    advance_test_time(&mut app, 1.49);
    app.update();

    assert_eq!(
        app.world().get::<CombatStats>(player).unwrap().current_health,
        100.0
    );

    advance_test_time(&mut app, 0.02);
    app.update();

    assert_eq!(
        app.world().get::<CombatStats>(player).unwrap().current_health,
        83.0
    );
}

#[test]
fn enemy_attack_uses_health_pool_when_present() {
    let mut app = setup_enemy_attack_test_app();

    let player = spawn_player_with_health_pool(
        app.world_mut(),
        Vec3::new(0.0, 0.0, 1.0),
        100.0,
        100.0,
        100.0,
        100.0,
    );

    app.world_mut().spawn((
        Enemy,
        EnemyAi {
            target: Some(player),
            ..EnemyAi::default()
        },
        test_enemy_attack_state(player),
        Transform::default(),
    ));

    advance_test_time(&mut app, DEFAULT_ENEMY_ATTACK_INTERVAL);
    app.update();

    let health_pool = app.world().get::<HealthPool>(player).unwrap();
    let combat_stats = app.world().get::<CombatStats>(player).unwrap();

    assert_eq!(
        health_pool.current(),
        100.0 - DEFAULT_ENEMY_ATTACK_DAMAGE
    );

    assert_eq!(
        combat_stats.current_health,
        health_pool.current()
    );
}

#[test]
fn enemy_attack_caps_damage_at_remaining_health_pool() {
    let mut app = setup_enemy_attack_test_app();

    let player = spawn_player_with_health_pool(
        app.world_mut(),
        Vec3::new(0.0, 0.0, 1.0),
        3.0,
        100.0,
        3.0,
        100.0,
    );

    app.world_mut().spawn((
        Enemy,
        EnemyAi {
            target: Some(player),
            ..EnemyAi::default()
        },
        test_enemy_attack_state(player),
        Transform::default(),
    ));

    advance_test_time(&mut app, DEFAULT_ENEMY_ATTACK_INTERVAL);
    app.update();

    let health_pool = app.world().get::<HealthPool>(player).unwrap();
    let combat_stats = app.world().get::<CombatStats>(player).unwrap();

    assert_eq!(health_pool.current(), 0.0);
    assert_eq!(combat_stats.current_health, 0.0);
}

#[test]
fn enemy_attack_caps_damage_at_remaining_ally_summon_health() {
    let mut app = setup_enemy_attack_test_app();

    let ally_summon = spawn_living_ally_summon(
        app.world_mut(),
        Vec3::new(0.0, 0.0, 1.0),
        3.0,
        100.0,
        3.0,
        100.0,
    );

    app.world_mut().spawn((
        Enemy,
        EnemyAi {
            target: Some(ally_summon),
            ..EnemyAi::default()
        },
        test_enemy_attack_state(ally_summon),
        Transform::default(),
    ));

    advance_test_time(&mut app, DEFAULT_ENEMY_ATTACK_INTERVAL);
    app.update();

    let health_pool = app.world().get::<HealthPool>(ally_summon).unwrap();
    let combat_stats = app.world().get::<CombatStats>(ally_summon).unwrap();

    assert_eq!(health_pool.current(), 0.0);
    assert_eq!(combat_stats.current_health, 0.0);
    assert_eq!(combat_stats.max_health, 100.0);
}

#[test]
fn enemy_attack_uses_health_pool_liveness_when_stats_are_stale() {
    let mut app = setup_enemy_attack_test_app();

    let player = spawn_player_with_health_pool(
        app.world_mut(),
        Vec3::new(0.0, 0.0, 1.0),
        0.0,
        50.0,
        100.0,
        120.0,
    );

    let enemy = app.world_mut().spawn((
        Enemy,
        EnemyAi {
            target: Some(player),
            ..EnemyAi::default()
        },
        test_enemy_attack_state(player),
        Transform::default(),
    )).id();

    advance_test_time(&mut app, DEFAULT_ENEMY_ATTACK_INTERVAL);
    app.update();

    let health_pool = app.world().get::<HealthPool>(player).unwrap();
    let combat_stats = app.world().get::<CombatStats>(player).unwrap();

    assert_eq!(
        health_pool.current(),
        100.0 - DEFAULT_ENEMY_ATTACK_DAMAGE
    );

    assert_eq!(
        combat_stats.current_health,
        health_pool.current()
    );

    assert_eq!(combat_stats.max_health, 120.0);

    assert_eq!(
        app.world().get::<EnemyAttackState>(enemy).unwrap().target,
        player
    );
}

#[test]
fn enemy_attack_resets_cooldown_when_health_pool_is_depleted() {
    let mut app = setup_enemy_attack_test_app();

    let player = spawn_player_with_health_pool(
        app.world_mut(),
        Vec3::new(0.0, 0.0, 1.0),
        100.0,
        100.0,
        0.0,
        120.0,
    );

    let enemy = app.world_mut().spawn((
        Enemy,
        EnemyAi {
            target: Some(player),
            ..EnemyAi::default()
        },
        test_enemy_attack_state(player),
        Transform::default(),
    )).id();

    advance_test_time(&mut app, DEFAULT_ENEMY_ATTACK_INTERVAL);
    app.update();

    let health_pool = app.world().get::<HealthPool>(player).unwrap();
    let combat_stats = app.world().get::<CombatStats>(player).unwrap();
    let attack_state = app.world().get::<EnemyAttackState>(enemy).unwrap();

    assert_eq!(health_pool.current(), 0.0);
    assert_eq!(combat_stats.current_health, 0.0);
    assert_eq!(combat_stats.max_health, 120.0);
    assert_eq!(attack_state.cooldown.elapsed(), Duration::ZERO);
}
