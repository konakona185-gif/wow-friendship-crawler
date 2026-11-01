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
