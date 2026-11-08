
use bevy::prelude::*;

use super::*;

pub(super) fn enemy_death_system(
    mut commands: Commands,
    enemy_query: Query<
        (Entity, Option<&CombatStats>, Option<&HealthPool>),
        With<Enemy>,
    >,
    indicator_query: Query<(Entity, &TargetIndicator)>,
    mut player_target_query: Query<&mut Target, With<Character>>,
) {
    for (enemy_entity, stats, health_pool) in &enemy_query {
        let current_health = health_pool
            .map(|pool| pool.current())
            .or_else(|| stats.map(|stats| stats.current_health));

        let Some(current_health) = current_health else {
            continue;
        };

        if current_health <= 0.0 {
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
mod tests;
