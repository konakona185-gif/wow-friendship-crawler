use super::*;

#[test]
fn defeating_an_enemy_removes_its_indicator_and_clears_target() {
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
