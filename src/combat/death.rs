use bevy::prelude::*;

use super::*;

pub(super) fn enemy_death_system(
    mut commands: Commands,
    enemy_query: Query<(Entity, &CombatStats), With<Enemy>>,
    indicator_query: Query<(Entity, &TargetIndicator)>,
    mut player_target_query: Query<&mut Target, With<Character>>,
) {
    for (enemy_entity, stats) in &enemy_query {
        if stats.current_health <= 0.0 {
            info!(
                "Nemico {:?} sconfitto. Rimozione dal mondo ECS.",
                enemy_entity
            );

            commands.entity(enemy_entity).despawn();

            for (indicator_entity, indicator) in &indicator_query {
                if indicator.enemy == enemy_entity {
                    commands.entity(indicator_entity).despawn();
                }
            }

            for mut player_target in &mut player_target_query {
                if player_target.entity == Some(enemy_entity) {
                    player_target.entity = None;
                    info!("Target del giocatore resettato.");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
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
}
