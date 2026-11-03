
use super::*;

#[test]
fn health_pool_defeated_enemy_removes_its_indicator_and_clears_target() {
    let mut app = App::new();
    app.add_systems(Update, enemy_death_system);

    let enemy = app.world_mut().spawn((
        Enemy,
        CombatStats {
            current_health: 80.0,
            max_health: 100.0,
            resource: 0.0,
        },
        HealthPool::new(0.0, 100.0).unwrap(),
    )).id();

    let indicator = app.world_mut().spawn(TargetIndicator {
        enemy,
    }).id();

    let player = app.world_mut().spawn((
        Character,
        Target {
            entity: Some(enemy),
        },
    )).id();

    app.update();

    assert!(app.world().get_entity(enemy).is_err());
    assert!(app.world().get_entity(indicator).is_err());

    assert_eq!(
        app.world().get::<Target>(player).unwrap().entity,
        None
    );
}

#[test]
fn health_pool_only_can_remove_defeated_enemy() {
    let mut app = App::new();
    app.add_systems(Update, enemy_death_system);

    let enemy = app.world_mut().spawn((
        Enemy,
        HealthPool::new(0.0, 100.0).unwrap(),
    )).id();

    let indicator = app.world_mut().spawn(TargetIndicator {
        enemy,
    }).id();

    let player = app.world_mut().spawn((
        Character,
        Target {
            entity: Some(enemy),
        },
    )).id();

    app.update();

    assert!(app.world().get_entity(enemy).is_err());
    assert!(app.world().get_entity(indicator).is_err());

    assert_eq!(
        app.world().get::<Target>(player).unwrap().entity,
        None
    );
}

#[test]
fn living_health_pool_prevents_enemy_removal_when_combat_stats_are_zero() {
    let mut app = App::new();
    app.add_systems(Update, enemy_death_system);

    let enemy = app.world_mut().spawn((
        Enemy,
        CombatStats {
            current_health: 0.0,
            max_health: 100.0,
            resource: 0.0,
        },
        HealthPool::new(50.0, 100.0).unwrap(),
    )).id();

    let indicator = app.world_mut().spawn(TargetIndicator {
        enemy,
    }).id();

    let player = app.world_mut().spawn((
        Character,
        Target {
            entity: Some(enemy),
        },
    )).id();

    app.update();

    assert!(app.world().get_entity(enemy).is_ok());
    assert!(app.world().get_entity(indicator).is_ok());

    assert_eq!(
        app.world().get::<Target>(player).unwrap().entity,
        Some(enemy)
    );
}

#[test]
fn combat_stats_remains_death_fallback_without_health_pool() {
    let mut app = App::new();
    app.add_systems(Update, enemy_death_system);

    let enemy = app.world_mut().spawn((
        Enemy,
        CombatStats {
            current_health: 0.0,
            max_health: 100.0,
            resource: 0.0,
        },
    )).id();

    let indicator = app.world_mut().spawn(TargetIndicator {
        enemy,
    }).id();

    let player = app.world_mut().spawn((
        Character,
        Target {
            entity: Some(enemy),
        },
    )).id();

    app.update();

    assert!(app.world().get_entity(enemy).is_err());
    assert!(app.world().get_entity(indicator).is_err());

    assert_eq!(
        app.world().get::<Target>(player).unwrap().entity,
        None
    );
}

#[test]
fn enemy_without_health_components_is_not_removed() {
    let mut app = App::new();
    app.add_systems(Update, enemy_death_system);

    let enemy = app.world_mut().spawn(Enemy).id();

    let indicator = app.world_mut().spawn(TargetIndicator {
        enemy,
    }).id();

    let player = app.world_mut().spawn((
        Character,
        Target {
            entity: Some(enemy),
        },
    )).id();

    app.update();

    assert!(app.world().get_entity(enemy).is_ok());
    assert!(app.world().get_entity(indicator).is_ok());

    assert_eq!(
        app.world().get::<Target>(player).unwrap().entity,
        Some(enemy)
    );
}
