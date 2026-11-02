use bevy::prelude::*;

use crate::interaction::WorldObjectClicked;

use super::*;

pub(super) fn target_tab_system(
    mut player_query: Query<
        (&Transform, &mut Target, &mut CharacterCommand),
        With<Character>,
    >,
    enemy_query: Query<
        (Entity, &Transform, Option<&CombatStats>, Option<&HealthPool>),
        With<Enemy>,
    >,
) {
    for (player_transform, mut player_target, mut command) in &mut player_query {
        if !std::mem::take(&mut command.select_target) {
            continue;
        }

        let mut enemies: Vec<(Entity, f32)> = enemy_query
            .iter()
            .filter_map(|(entity, transform, stats, health_pool)| {
                let current_health = health_pool
                    .map(|pool| pool.current())
                    .or_else(|| stats.map(|stats| stats.current_health));

                if current_health.is_some_and(|health| health <= 0.0) {
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
    enemy_query: Query<
        (Option<&CombatStats>, Option<&HealthPool>),
        With<Enemy>,
    >,
    mut player_query: Query<&mut Target, With<Character>>,
) {
    for click in clicked.read() {
        if click.button != MouseButton::Left {
            continue;
        }

        let Some(entity) = click.entity else {
            continue;
        };

        let Ok((stats, health_pool)) = enemy_query.get(entity) else {
            continue;
        };

        let current_health = health_pool
            .map(|pool| pool.current())
            .or_else(|| stats.map(|stats| stats.current_health));

        if current_health.is_some_and(|health| health <= 0.0) {
            continue;
        }

        for mut target in &mut player_query {
            target.entity = Some(entity);
            info!("Target selezionato tramite click: {:?}", entity);
        }
    }
}

#[cfg(test)]
mod tests;
