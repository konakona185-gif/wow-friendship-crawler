use super::*;

#[test]
fn target_indicator_is_visible_only_for_the_selected_enemy() {
    let mut app = App::new();
    app.add_systems(Update, target_indicator_system);

    let selected_position = Vec3::new(4.0, 2.0, -3.0);
    let selected_enemy = app.world_mut().spawn((
        Enemy,
        Transform::from_translation(selected_position),
    )).id();

    let other_enemy = app.world_mut().spawn((
        Enemy,
        Transform::from_xyz(8.0, 0.0, 1.0),
    )).id();

    app.world_mut().spawn((
        Character,
        Target {
            entity: Some(selected_enemy),
        },
    ));

    let selected_indicator = app.world_mut().spawn((
        TargetIndicator {
            enemy: selected_enemy,
        },
        Transform::default(),
        Visibility::Hidden,
    )).id();

    let other_indicator = app.world_mut().spawn((
        TargetIndicator {
            enemy: other_enemy,
        },
        Transform::default(),
        Visibility::Hidden,
    )).id();

    app.update();

    assert_eq!(
        app.world().get::<Visibility>(selected_indicator),
        Some(&Visibility::Visible)
    );

    assert_eq!(
        app.world().get::<Visibility>(other_indicator),
        Some(&Visibility::Hidden)
    );

    let expected_position =
        selected_position + Vec3::new(0.0, -0.47, 0.0);

    let actual_position = app
        .world()
        .get::<Transform>(selected_indicator)
        .unwrap()
        .translation;

    assert!((actual_position - expected_position).length() < 1.0e-6);
}
