use super::*;

#[test]
fn tab_selects_the_nearest_enemy() {
    let mut app = setup_target_test_app();

    let player = app.world_mut().spawn((
        Character,
        Transform::from_translation(Vec3::ZERO),
        Target { entity: None },
        CharacterCommand {
            select_target: true,
            ..default()
        },
    )).id();

    app.world_mut().spawn((
        Enemy,
        Transform::from_xyz(0.0, 0.0, 8.0),
    ));

    let nearest_enemy = app.world_mut().spawn((
        Enemy,
        Transform::from_xyz(0.0, 0.0, 3.0),
    )).id();

    app.update();

    assert_eq!(
        app.world().get::<Target>(player).unwrap().entity,
        Some(nearest_enemy)
    );

    assert!(
        !app.world()
            .get::<CharacterCommand>(player)
            .unwrap()
            .select_target
    );
}

#[test]
fn tab_does_not_select_defeated_enemies() {
    let mut app = setup_target_test_app();

    let player = app.world_mut().spawn((
        Character,
        Transform::default(),
        Target { entity: None },
        CharacterCommand {
            select_target: true,
            ..default()
        },
    )).id();

    app.world_mut().spawn((
        Enemy,
        Transform::from_xyz(0.0, 0.0, 1.0),
        CombatStats {
            current_health: 0.0,
            max_health: 100.0,
            resource: 0.0,
        },
    ));

    let living_enemy = app.world_mut().spawn((
        Enemy,
        Transform::from_xyz(0.0, 0.0, 5.0),
        CombatStats {
            current_health: 100.0,
            max_health: 100.0,
            resource: 0.0,
        },
    )).id();

    app.update();

    assert_eq!(
        app.world().get::<Target>(player).unwrap().entity,
        Some(living_enemy)
    );
}

#[test]
fn tab_uses_health_pool_to_exclude_defeated_enemies() {
    let mut app = setup_target_test_app();

    let player = app.world_mut().spawn((
        Character,
        Transform::default(),
        Target { entity: None },
        CharacterCommand {
            select_target: true,
            ..default()
        },
    )).id();

    app.world_mut().spawn((
        Enemy,
        Transform::from_xyz(0.0, 0.0, 1.0),
        CombatStats {
            current_health: 100.0,
            max_health: 100.0,
            resource: 0.0,
        },
        HealthPool::new(0.0, 100.0).unwrap(),
    ));

    let living_enemy = app.world_mut().spawn((
        Enemy,
        Transform::from_xyz(0.0, 0.0, 5.0),
        CombatStats {
            current_health: 100.0,
            max_health: 100.0,
            resource: 0.0,
        },
        HealthPool::new(50.0, 100.0).unwrap(),
    )).id();

    app.update();

    assert_eq!(
        app.world().get::<Target>(player).unwrap().entity,
        Some(living_enemy)
    );
}

#[test]
fn tab_selects_enemy_with_living_health_pool_when_combat_stats_are_zero() {
    let mut app = setup_target_test_app();

    let player = app.world_mut().spawn((
        Character,
        Transform::default(),
        Target { entity: None },
        CharacterCommand {
            select_target: true,
            ..default()
        },
    )).id();

    let living_enemy = app.world_mut().spawn((
        Enemy,
        Transform::from_xyz(0.0, 0.0, 3.0),
        CombatStats {
            current_health: 0.0,
            max_health: 100.0,
            resource: 0.0,
        },
        HealthPool::new(50.0, 100.0).unwrap(),
    )).id();

    app.update();

    assert_eq!(
        app.world().get::<Target>(player).unwrap().entity,
        Some(living_enemy)
    );
}

#[test]
fn tab_selects_enemy_with_only_a_living_health_pool() {
    let mut app = setup_target_test_app();

    let player = app.world_mut().spawn((
        Character,
        Transform::default(),
        Target { entity: None },
        CharacterCommand {
            select_target: true,
            ..default()
        },
    )).id();

    let living_enemy = app.world_mut().spawn((
        Enemy,
        Transform::from_xyz(0.0, 0.0, 3.0),
        HealthPool::new(50.0, 100.0).unwrap(),
    )).id();

    app.update();

    assert_eq!(
        app.world().get::<Target>(player).unwrap().entity,
        Some(living_enemy)
    );
}

#[test]
fn tab_cycles_through_enemies_by_distance_and_wraps() {
    let mut app = setup_target_test_app();

    let player = app.world_mut().spawn((
        Character,
        Transform::default(),
        Target { entity: None },
        CharacterCommand {
            select_target: true,
            ..default()
        },
    )).id();

    let nearest = app.world_mut().spawn((
        Enemy,
        Transform::from_xyz(0.0, 0.0, 3.0),
    )).id();

    let middle = app.world_mut().spawn((
        Enemy,
        Transform::from_xyz(0.0, 0.0, 8.0),
    )).id();

    let furthest = app.world_mut().spawn((
        Enemy,
        Transform::from_xyz(0.0, 0.0, 13.0),
    )).id();

    app.update();

    assert_eq!(
        app.world().get::<Target>(player).unwrap().entity,
        Some(nearest)
    );

    for expected in [middle, furthest, nearest] {
        app.world_mut()
            .get_mut::<CharacterCommand>(player)
            .unwrap()
            .select_target = true;

        app.update();

        assert_eq!(
            app.world().get::<Target>(player).unwrap().entity,
            Some(expected)
        );
    }
}

#[test]
fn tab_clears_target_when_no_enemies_exist() {
    let mut app = setup_target_test_app();

    let stale_target = app.world_mut().spawn_empty().id();

    let player = app.world_mut().spawn((
        Character,
        Transform::default(),
        Target {
            entity: Some(stale_target),
        },
        CharacterCommand {
            select_target: true,
            ..default()
        },
    )).id();

    app.update();

    assert_eq!(
        app.world().get::<Target>(player).unwrap().entity,
        None
    );
}
