use bevy::prelude::*;

use super::*;

fn combatant_is_alive(
    stats: Option<&CombatStats>,
    health_pool: Option<&HealthPool>,
) -> bool {
    health_pool
        .map(|pool| pool.current())
        .or_else(|| stats.map(|stats| stats.current_health))
        .is_some_and(|health| health > 0.0)
}

pub(super) fn enemy_aggro_system(
    ally_query: Query<
        (
            Entity,
            &Transform,
            Option<&CombatStats>,
            Option<&HealthPool>,
        ),
        (
            Or<(With<Character>, With<AllySummon>)>,
            Without<Enemy>,
        ),
    >,
    mut enemy_query: Query<
        (
            &Transform,
            Option<&mut CombatStats>,
            &mut EnemyAi,
            Option<&EnemyHomePosition>,
            Option<&mut EnemyThreat>,
            Option<&mut HealthPool>,
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
        health_pool,
    ) in &mut enemy_query
    {
        let current_health = health_pool
            .as_deref()
            .map(|pool| pool.current())
            .or_else(|| {
                enemy_stats
                    .as_deref()
                    .map(|stats| stats.current_health)
            });

        let Some(current_health) = current_health else {
            continue;
        };

        if let Some(pool) = health_pool.as_deref() {
            if let Some(stats) = enemy_stats.as_deref_mut() {
                stats.current_health = pool.current();
                stats.max_health = pool.maximum();
            }
        }

        if current_health <= 0.0 {
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

                    let restored_health = health_pool
                        .as_deref()
                        .map(|pool| pool.maximum())
                        .or_else(|| {
                            enemy_stats
                                .as_deref()
                                .map(|stats| stats.max_health)
                        })
                        .unwrap_or(0.0)
                        .max(0.0);

                    if let Some(stats) = enemy_stats.as_deref_mut() {
                        stats.current_health = restored_health;
                    }

                    if let Some(mut pool) = health_pool {
                        if let Err(error) = pool.set_current(restored_health) {
                            warn!(
                                "Impossibile ripristinare la HealthPool del nemico: {error}"
                            );
                        }
                    }

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

        let highest_threat_target =
            if let Some(threat) = enemy_threat.as_deref_mut() {
                threat.values.retain(|ally_entity, value| {
                    value.is_finite()
                        && *value > 0.0
                        && ally_query
                            .get(*ally_entity)
                            .is_ok_and(|(_, _, stats, health_pool)| {
                                combatant_is_alive(stats, health_pool)
                            })
                });

                threat.values
                    .iter()
                    .filter_map(|(ally_entity, value)| {
                        let (_, ally_transform, stats, health_pool) =
                            ally_query.get(*ally_entity).ok()?;

                        combatant_is_alive(stats, health_pool)
                            .then_some((
                                *ally_entity,
                                *value,
                                enemy_transform
                                    .translation
                                    .distance_squared(
                                        ally_transform.translation,
                                    ),
                            ))
                    })
                    .max_by(|a, b| {
                        a.1.total_cmp(&b.1)
                            .then_with(|| b.2.total_cmp(&a.2))
                    })
                    .map(|(ally_entity, _, _)| ally_entity)
            } else {
                None
            };

        if let Some(target_entity) = highest_threat_target {
            enemy_ai.target = Some(target_entity);
            continue;
        }

        let current_target_is_alive = enemy_ai.target.is_some_and(
            |target_entity| {
                ally_query
                    .get(target_entity)
                    .is_ok_and(|(_, _, stats, health_pool)| {
                        combatant_is_alive(stats, health_pool)
                    })
            },
        );

        if current_target_is_alive {
            continue;
        }

        let aggro_range = if enemy_ai.aggro_range.is_finite() {
            enemy_ai.aggro_range.max(0.0)
        } else {
            DEFAULT_ENEMY_AGGRO_RANGE
        };

        let aggro_range_squared = aggro_range * aggro_range;

        let new_target = ally_query
            .iter()
            .filter(|(_, _, stats, health_pool)| {
                combatant_is_alive(*stats, *health_pool)
            })
            .filter_map(|(entity, ally_transform, _, _)| {
                let distance_squared = enemy_transform
                    .translation
                    .distance_squared(ally_transform.translation);

                (distance_squared <= aggro_range_squared)
                    .then_some((entity, distance_squared))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(entity, _)| entity);

        enemy_ai.target = new_target;

        if let Some(target_entity) = new_target {
            info!(
                "Nemico ha acquisito un alleato {:?} come bersaglio.",
                target_entity
            );
        }
    }
}

#[cfg(test)]
mod tests;
