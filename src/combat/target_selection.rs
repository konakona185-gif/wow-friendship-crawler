use bevy::prelude::*;

use crate::interaction::WorldObjectClicked;

use super::*;

pub(super) fn target_tab_system(
    mut player_query: Query<
        (&Transform, &mut Target, &mut CharacterCommand),
        With<Character>,
    >,
    enemy_query: Query<
        (Entity, &Transform, Option<&CombatStats>),
        With<Enemy>,
    >,
) {
    for (player_transform, mut player_target, mut command) in &mut player_query {
        if !std::mem::take(&mut command.select_target) {
            continue;
        }

        let mut enemies: Vec<(Entity, f32)> = enemy_query
            .iter()
            .filter_map(|(entity, transform, stats)| {
                if stats.is_some_and(|stats| stats.current_health <= 0.0) {
                    return None;
                }

                let distance_squared = player_transform
                    .translation
                    .distance_squared(transform.translation);

                Some((entity, distance_squared))
            })
            .collect();

        enemies.sort_by(|a, b| a.1.total_cmp(&b.1));

        if enemies.is_empty() {
            player_target.entity = None;
            continue;
        }

        let next_index = player_target
            .entity
            .and_then(|current| {
                enemies.iter().position(|(entity, _)| *entity == current)
            })
            .map_or(0, |index| (index + 1) % enemies.len());

        let enemy_entity = enemies[next_index].0;

        player_target.entity = Some(enemy_entity);

        info!("Target selezionato: {:?}", enemy_entity);
    }
}

pub(super) fn target_click_system(
    mut clicked: MessageReader<WorldObjectClicked>,
    enemy_query: Query<Option<&CombatStats>, With<Enemy>>,
    mut player_query: Query<&mut Target, With<Character>>,
) {
    for click in clicked.read() {
        if click.button != MouseButton::Left {
            continue;
        }

        let Some(entity) = click.entity else {
            continue;
        };

        let Ok(stats) = enemy_query.get(entity) else {
            continue;
        };

        if stats.is_some_and(|stats| stats.current_health <= 0.0) {
            continue;
        }

        for mut target in &mut player_query {
            target.entity = Some(entity);
            info!("Target selezionato tramite click: {:?}", entity);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use benilla_world::interact::WorldObject;
    use benilla_world::model_render::ModelKind;

    fn setup_target_test_app() -> App {
        let mut app = App::new();

        app.add_message::<WorldObjectClicked>();
        app.add_systems(
            Update,
            (target_tab_system, target_click_system).chain(),
        );

        app
    }

    fn test_world_object() -> WorldObject {
        WorldObject {
            kind: ModelKind::Creature,
            label: "Test creature".to_string(),
            id: 1,
            detail: String::new(),
        }
    }

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

    #[test]
    fn left_click_selects_the_exact_clicked_enemy() {
        let mut app = setup_target_test_app();

        let player = app.world_mut().spawn((
            Character,
            Transform::default(),
            Target { entity: None },
            CharacterCommand::default(),
        )).id();

        app.world_mut().spawn((
            Enemy,
            Transform::from_xyz(0.0, 0.0, 2.0),
        ));

        let clicked_enemy = app.world_mut().spawn((
            Enemy,
            Transform::from_xyz(0.0, 0.0, 12.0),
        )).id();

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

        let player = app.world_mut().spawn((
            Character,
            Transform::default(),
            Target { entity: None },
            CharacterCommand::default(),
        )).id();

        let defeated_enemy = app.world_mut().spawn((
            Enemy,
            CombatStats {
                current_health: 0.0,
                max_health: 100.0,
                resource: 0.0,
            },
            Transform::from_xyz(0.0, 0.0, 2.0),
        )).id();

        app.world_mut().write_message(WorldObjectClicked {
            button: MouseButton::Left,
            entity: Some(defeated_enemy),
            object: test_world_object(),
            point: Vec3::ZERO,
            distance: 2.0,
        });

        app.update();

        assert_eq!(
            app.world().get::<Target>(player).unwrap().entity,
            None
        );
    }

    #[test]
    fn right_click_and_clicking_non_enemies_do_not_change_target() {
        let mut app = setup_target_test_app();

        let player = app.world_mut().spawn((
            Character,
            Transform::default(),
            Target { entity: None },
            CharacterCommand::default(),
        )).id();

        let enemy = app.world_mut().spawn((
            Enemy,
            Transform::from_xyz(0.0, 0.0, 2.0),
        )).id();

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

        assert_eq!(
            app.world().get::<Target>(player).unwrap().entity,
            None
        );
    }
}
