use super::*;

#[test]
fn left_click_selects_the_exact_clicked_enemy() {
    let mut app = setup_target_test_app();

    let player = app
        .world_mut()
        .spawn((
            Character,
            Transform::default(),
            Target { entity: None },
            CharacterCommand::default(),
        ))
        .id();

    app.world_mut()
        .spawn((Enemy, Transform::from_xyz(0.0, 0.0, 2.0)));

    let clicked_enemy = app
        .world_mut()
        .spawn((Enemy, Transform::from_xyz(0.0, 0.0, 12.0)))
        .id();

    app.world_mut().write_message(WorldObjectClicked {
        button: MouseButton::Left,
        entity: Some(clicked_enemy),
        object: test_world_object(),
        point: Vec3::ZERO,
        distance: 12.0,
    });

    app.update();

    assert_eq!(
        app.world().get::<Target>(player).unwrap().entity,
        Some(clicked_enemy)
    );
}

#[test]
fn clicking_a_defeated_enemy_does_not_change_target() {
    let mut app = setup_target_test_app();

    let player = app
        .world_mut()
        .spawn((
            Character,
            Transform::default(),
            Target { entity: None },
            CharacterCommand::default(),
        ))
        .id();

    let defeated_enemy = app
        .world_mut()
        .spawn((
            Enemy,
            CombatStats {
                current_health: 0.0,
                max_health: 100.0,
                resource: 0.0,
            },
            Transform::from_xyz(0.0, 0.0, 2.0),
        ))
        .id();

    app.world_mut().write_message(WorldObjectClicked {
        button: MouseButton::Left,
        entity: Some(defeated_enemy),
        object: test_world_object(),
        point: Vec3::ZERO,
        distance: 2.0,
    });

    app.update();

    assert_eq!(app.world().get::<Target>(player).unwrap().entity, None);
}

#[test]
fn clicking_an_enemy_with_zero_health_pool_does_not_change_target() {
    let mut app = setup_target_test_app();

    let player = app
        .world_mut()
        .spawn((
            Character,
            Transform::default(),
            Target { entity: None },
            CharacterCommand::default(),
        ))
        .id();

    let defeated_enemy = app
        .world_mut()
        .spawn((
            Enemy,
            CombatStats {
                current_health: 100.0,
                max_health: 100.0,
                resource: 0.0,
            },
            HealthPool::new(0.0, 100.0).unwrap(),
            Transform::from_xyz(0.0, 0.0, 2.0),
        ))
        .id();

    app.world_mut().write_message(WorldObjectClicked {
        button: MouseButton::Left,
        entity: Some(defeated_enemy),
        object: test_world_object(),
        point: Vec3::ZERO,
        distance: 2.0,
    });

    app.update();

    assert_eq!(app.world().get::<Target>(player).unwrap().entity, None);
}

#[test]
fn clicking_an_enemy_with_living_health_pool_selects_it_when_combat_stats_are_zero() {
    let mut app = setup_target_test_app();

    let player = app
        .world_mut()
        .spawn((
            Character,
            Transform::default(),
            Target { entity: None },
            CharacterCommand::default(),
        ))
        .id();

    let living_enemy = app
        .world_mut()
        .spawn((
            Enemy,
            CombatStats {
                current_health: 0.0,
                max_health: 100.0,
                resource: 0.0,
            },
            HealthPool::new(50.0, 100.0).unwrap(),
            Transform::from_xyz(0.0, 0.0, 2.0),
        ))
        .id();

    app.world_mut().write_message(WorldObjectClicked {
        button: MouseButton::Left,
        entity: Some(living_enemy),
        object: test_world_object(),
        point: Vec3::ZERO,
        distance: 2.0,
    });

    app.update();

    assert_eq!(
        app.world().get::<Target>(player).unwrap().entity,
        Some(living_enemy)
    );
}

#[test]
fn clicking_an_enemy_with_only_a_living_health_pool_selects_it() {
    let mut app = setup_target_test_app();

    let player = app
        .world_mut()
        .spawn((
            Character,
            Transform::default(),
            Target { entity: None },
            CharacterCommand::default(),
        ))
        .id();

    let living_enemy = app
        .world_mut()
        .spawn((
            Enemy,
            HealthPool::new(50.0, 100.0).unwrap(),
            Transform::from_xyz(0.0, 0.0, 2.0),
        ))
        .id();

    app.world_mut().write_message(WorldObjectClicked {
        button: MouseButton::Left,
        entity: Some(living_enemy),
        object: test_world_object(),
        point: Vec3::ZERO,
        distance: 2.0,
    });

    app.update();

    assert_eq!(
        app.world().get::<Target>(player).unwrap().entity,
        Some(living_enemy)
    );
}

#[test]
fn right_click_and_clicking_non_enemies_do_not_change_target() {
    let mut app = setup_target_test_app();

    let player = app
        .world_mut()
        .spawn((
            Character,
            Transform::default(),
            Target { entity: None },
            CharacterCommand::default(),
        ))
        .id();

    let enemy = app
        .world_mut()
        .spawn((Enemy, Transform::from_xyz(0.0, 0.0, 2.0)))
        .id();

    let non_enemy = app.world_mut().spawn_empty().id();

    app.world_mut().write_message(WorldObjectClicked {
        button: MouseButton::Right,
        entity: Some(enemy),
        object: test_world_object(),
        point: Vec3::ZERO,
        distance: 2.0,
    });

    app.world_mut().write_message(WorldObjectClicked {
        button: MouseButton::Left,
        entity: Some(non_enemy),
        object: test_world_object(),
        point: Vec3::ZERO,
        distance: 1.0,
    });

    app.update();

    assert_eq!(app.world().get::<Target>(player).unwrap().entity, None);
}
