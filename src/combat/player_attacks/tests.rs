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
