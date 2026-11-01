use bevy::prelude::*;

use super::*;

pub(super) fn enemy_aggro_system(
    player_query: Query<
        (Entity, &Transform, &CombatStats),
        (With<Character>, Without<Enemy>),
    >,
    mut enemy_query: Query<
        (
            &Transform,
            &mut CombatStats,
            &mut EnemyAi,
            Option<&EnemyHomePosition>,
            Option<&mut EnemyThreat>,
        ),
        (With<Enemy>, Without<Character>),
    >,
) {
    for (
        enemy_transform,
        mut enemy_stats,
        mut enemy_ai,
        home_position,
        mut enemy_threat,
    ) in &mut enemy_query
    {
        if enemy_stats.current_health <= 0.0 {
            enemy_ai.target = None;
            enemy_ai.returning_home = false;

            if let Some(threat) = enemy_threat.as_deref_mut() {
                threat.clear();
            }

            continue;
        }

        if let Some(home_position) = home_position {
            let distance_from_home_squared = enemy_transform
                .translation
                .distance_squared(home_position.0);

            let home_arrival_distance_squared =
                ENEMY_HOME_ARRIVAL_DISTANCE * ENEMY_HOME_ARRIVAL_DISTANCE;

            if enemy_ai.returning_home {
                if let Some(threat) = enemy_threat.as_deref_mut() {
                    threat.clear();
                }

                if distance_from_home_squared > home_arrival_distance_squared {
                    enemy_ai.target = None;
                    continue;
                }

                enemy_ai.returning_home = false;

                info!("Nemico tornato alla posizione iniziale.");
            } else {
                let leash_range = if enemy_ai.leash_range.is_finite() {
                    enemy_ai.leash_range.max(0.0)
                } else {
                    DEFAULT_ENEMY_LEASH_RANGE
                };

                let leash_range_squared = leash_range * leash_range;

                if distance_from_home_squared > leash_range_squared {
                    enemy_ai.target = None;
                    enemy_ai.returning_home = true;
                    enemy_stats.current_health =
                        enemy_stats.max_health.max(0.0);

                    if let Some(threat) = enemy_threat.as_deref_mut() {
                        threat.clear();
                    }

                    info!(
                        "Nemico fuori dal raggio di combattimento: ritorno alla posizione iniziale."
                    );

                    continue;
                }
            }
        }

        let highest_threat_target = if let Some(threat) = enemy_threat.as_deref_mut() {
            threat.values.retain(|player_entity, value| {
                value.is_finite()
                    && *value > 0.0
                    && player_query
                        .get(*player_entity)
                        .is_ok_and(|(_, _, stats)| stats.current_health > 0.0)
            });

            threat.values
                .iter()
                .filter_map(|(player_entity, value)| {
                    let (_, player_transform, stats) =
                        player_query.get(*player_entity).ok()?;

                    (stats.current_health > 0.0)
                        .then_some((
                            *player_entity,
                            *value,
                            enemy_transform
                                .translation
                                .distance_squared(player_transform.translation),
                        ))
                })
                .max_by(|a, b| {
                    a.1.total_cmp(&b.1)
                        .then_with(|| b.2.total_cmp(&a.2))
                })
                .map(|(player_entity, _, _)| player_entity)
        } else {
            None
        };

        if let Some(target_entity) = highest_threat_target {
            enemy_ai.target = Some(target_entity);
            continue;
        }

        let current_target_is_alive = enemy_ai.target.is_some_and(|target_entity| {
            player_query
                .get(target_entity)
                .is_ok_and(|(_, _, stats)| stats.current_health > 0.0)
        });

        if current_target_is_alive {
            continue;
        }

        let aggro_range = if enemy_ai.aggro_range.is_finite() {
            enemy_ai.aggro_range.max(0.0)
        } else {
            DEFAULT_ENEMY_AGGRO_RANGE
        };

        let aggro_range_squared = aggro_range * aggro_range;

        let new_target = player_query
            .iter()
            .filter(|(_, _, stats)| stats.current_health > 0.0)
            .filter_map(|(entity, player_transform, _)| {
                let distance_squared = enemy_transform
                    .translation
                    .distance_squared(player_transform.translation);

                (distance_squared <= aggro_range_squared)
                    .then_some((entity, distance_squared))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(entity, _)| entity);

        enemy_ai.target = new_target;

        if let Some(target_entity) = new_target {
            info!(
                "Nemico ha acquisito il giocatore {:?} come bersaglio.",
                target_entity
            );
        }
    }
}

#[cfg(test)]
mod tests;
