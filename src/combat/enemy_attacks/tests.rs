use super::*;
use std::time::Duration;

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
        cooldown: Timer::from_seconds(
            DEFAULT_ENEMY_ATTACK_INTERVAL,
            TimerMode::Repeating,
        ),
    }
}

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
    stats_maximum: f32,
    pool_health: f32,
    pool_maximum: f32,
) -> Entity {
    world.spawn((
        Character,
        Transform::from_translation(position),
        CombatStats {
            current_health: stats_health,
            max_health: stats_maximum,
            resource: 0.0,
        },
        HealthPool::new(pool_health, pool_maximum).unwrap(),
    )).id()
}

fn spawn_living_ally_summon(
    world: &mut World,
    position: Vec3,
    stats_health: f32,
    stats_maximum: f32,
    pool_health: f32,
    pool_maximum: f32,
) -> Entity {
    world.spawn((
        AllySummon,
        Transform::from_translation(position),
        CombatStats {
            current_health: stats_health,
            max_health: stats_maximum,
            resource: 0.0,
        },
        HealthPool::new(pool_health, pool_maximum).unwrap(),
    )).id()
}

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
