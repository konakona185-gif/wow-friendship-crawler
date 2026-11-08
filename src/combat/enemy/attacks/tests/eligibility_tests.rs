use super::*;

#[test]
fn enemy_attacks_a_living_player_after_the_melee_cooldown() {
    let mut app = setup_enemy_attack_test_app();

    let player = spawn_living_player(
        app.world_mut(),
        Vec3::new(0.0, 0.0, 1.0),
    );

    app.world_mut().spawn((
        Enemy,
        EnemyAi {
            target: Some(player),
            ..EnemyAi::default()
        },
        EnemyAttackState {
            target: player,
            cooldown: Timer::from_seconds(
                DEFAULT_ENEMY_ATTACK_INTERVAL,
                TimerMode::Repeating,
            ),
        },
        Transform::default(),
    ));

    advance_test_time(&mut app, DEFAULT_ENEMY_ATTACK_INTERVAL);
    app.update();

    assert_eq!(
        app.world().get::<CombatStats>(player).unwrap().current_health,
        100.0 - DEFAULT_ENEMY_ATTACK_DAMAGE
    );
}

#[test]
fn enemy_attacks_ally_summon_after_the_melee_cooldown() {
    let mut app = setup_enemy_attack_test_app();

    let ally_summon = spawn_living_ally_summon(
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
fn enemy_summon_uses_its_custom_melee_range() {
    let mut app = setup_enemy_attack_test_app();

    let player = spawn_living_player(
        app.world_mut(),
        Vec3::new(0.0, 0.0, 3.5),
    );

    spawn_enemy_summon(
        app.world_mut(),
        player,
        test_summon_combat_profile(
            18.0,
            1.0,
            4.0,
        ),
    );

    app.update();

    advance_test_time(&mut app, 1.0);
    app.update();

    assert_eq!(
        app.world().get::<CombatStats>(player).unwrap().current_health,
        82.0
    );
}

#[test]
fn enemy_summon_does_not_attack_outside_its_custom_melee_range() {
    let mut app = setup_enemy_attack_test_app();

    let player = spawn_living_player(
        app.world_mut(),
        Vec3::new(0.0, 0.0, 4.5),
    );

    spawn_enemy_summon(
        app.world_mut(),
        player,
        test_summon_combat_profile(
            18.0,
            1.0,
            4.0,
        ),
    );

    app.update();

    advance_test_time(&mut app, 1.0);
    app.update();

    assert_eq!(
        app.world().get::<CombatStats>(player).unwrap().current_health,
        100.0
    );
}

#[test]
fn enemy_attacks_ally_summon_with_only_health_pool() {
    let mut app = setup_enemy_attack_test_app();

    let ally_summon = app.world_mut().spawn((
        AllySummon,
        Transform::from_xyz(0.0, 0.0, 1.0),
        HealthPool::new(100.0, 120.0).unwrap(),
    )).id();

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

    assert_eq!(
        health_pool.current(),
        100.0 - DEFAULT_ENEMY_ATTACK_DAMAGE
    );
    assert_eq!(health_pool.maximum(), 120.0);
    assert!(app.world().get::<CombatStats>(ally_summon).is_none());
}

#[test]
fn enemy_attacks_player_with_only_health_pool() {
    let mut app = setup_enemy_attack_test_app();

    let player = app.world_mut().spawn((
        Character,
        Transform::from_xyz(0.0, 0.0, 1.0),
        HealthPool::new(100.0, 120.0).unwrap(),
    )).id();

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

    assert_eq!(
        health_pool.current(),
        100.0 - DEFAULT_ENEMY_ATTACK_DAMAGE
    );
    assert_eq!(health_pool.maximum(), 120.0);
    assert!(app.world().get::<CombatStats>(player).is_none());
}

#[test]
fn enemy_does_not_attack_player_without_health_components() {
    let mut app = setup_enemy_attack_test_app();

    let player = app.world_mut().spawn((
        Character,
        Transform::from_xyz(0.0, 0.0, 1.0),
    )).id();

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

    let attack_state = app.world().get::<EnemyAttackState>(enemy).unwrap();

    assert_eq!(attack_state.cooldown.elapsed(), Duration::ZERO);
    assert!(app.world().get::<HealthPool>(player).is_none());
    assert!(app.world().get::<CombatStats>(player).is_none());
}

#[test]
fn enemy_does_not_attack_outside_melee_range() {
    let mut app = setup_enemy_attack_test_app();

    let player = spawn_living_player(
        app.world_mut(),
        Vec3::new(0.0, 0.0, DEFAULT_ENEMY_MELEE_RANGE + 1.0),
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

    assert_eq!(
        app.world().get::<CombatStats>(player).unwrap().current_health,
        100.0
    );

    app.world_mut()
        .get_mut::<Transform>(player)
        .unwrap()
        .translation = Vec3::new(0.0, 0.0, 1.0);

    advance_test_time(&mut app, DEFAULT_ENEMY_ATTACK_INTERVAL - 0.1);
    app.update();

    assert_eq!(
        app.world().get::<CombatStats>(player).unwrap().current_health,
        100.0
    );

    advance_test_time(&mut app, 0.11);
    app.update();

    assert_eq!(
        app.world().get::<CombatStats>(player).unwrap().current_health,
        100.0 - DEFAULT_ENEMY_ATTACK_DAMAGE
    );
}

#[test]
fn returning_enemies_do_not_attack_players() {
    let mut app = setup_enemy_attack_test_app();

    let player = spawn_living_player(
        app.world_mut(),
        Vec3::new(0.0, 0.0, 1.0),
    );

    app.world_mut().spawn((
        Enemy,
        EnemyAi {
            target: Some(player),
            returning_home: true,
            ..EnemyAi::default()
        },
        test_enemy_attack_state(player),
        Transform::default(),
    ));

    advance_test_time(&mut app, DEFAULT_ENEMY_ATTACK_INTERVAL);
    app.update();

    assert_eq!(
        app.world().get::<CombatStats>(player).unwrap().current_health,
        100.0
    );
}

#[test]
fn defeated_players_cannot_be_damaged_by_enemy_attacks() {
    let mut app = setup_enemy_attack_test_app();

    let player = app.world_mut().spawn((
        Character,
        Transform::from_xyz(0.0, 0.0, 1.0),
        CombatStats {
            current_health: 0.0,
            max_health: 100.0,
            resource: 0.0,
        },
    )).id();

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

    assert_eq!(
        app.world().get::<CombatStats>(player).unwrap().current_health,
        0.0
    );
}

#[test]
fn defeated_ally_summons_cannot_be_damaged_by_enemy_attacks() {
    let mut app = setup_enemy_attack_test_app();

    let ally_summon = spawn_living_ally_summon(
        app.world_mut(),
        Vec3::new(0.0, 0.0, 1.0),
        100.0,
        100.0,
        0.0,
        100.0,
    );

    let enemy = app.world_mut().spawn((
        Enemy,
        EnemyAi {
            target: Some(ally_summon),
            ..EnemyAi::default()
        },
        test_enemy_attack_state(ally_summon),
        Transform::default(),
    )).id();

    advance_test_time(&mut app, DEFAULT_ENEMY_ATTACK_INTERVAL);
    app.update();

    let health_pool = app.world().get::<HealthPool>(ally_summon).unwrap();
    let combat_stats = app.world().get::<CombatStats>(ally_summon).unwrap();
    let attack_state = app.world().get::<EnemyAttackState>(enemy).unwrap();

    assert_eq!(health_pool.current(), 0.0);
    assert_eq!(combat_stats.current_health, 0.0);
    assert_eq!(attack_state.cooldown.elapsed(), Duration::ZERO);
}
