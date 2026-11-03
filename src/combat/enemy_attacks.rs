
use bevy::prelude::*;

use super::*;

#[derive(Component)]
pub(super) struct EnemyAttackState {
    target: Entity,
    cooldown: Timer,
}

pub(super) fn enemy_attack_system(
    time: Res<Time>,
    mut commands: Commands,
    mut enemy_query: Query<
        (
            Entity,
            &Transform,
            &EnemyAi,
            Option<&mut EnemyAttackState>,
        ),
        With<Enemy>,
    >,
    mut player_query: Query<
        (&Transform, Option<&mut CombatStats>, Option<&mut HealthPool>),
        (With<Character>, Without<Enemy>),
    >,
) {
    let melee_range_squared =
        DEFAULT_ENEMY_MELEE_RANGE * DEFAULT_ENEMY_MELEE_RANGE;

    for (enemy_entity, enemy_transform, enemy_ai, attack_state) in
        &mut enemy_query
    {
        if enemy_ai.returning_home {
            if let Some(mut state) = attack_state {
                state.cooldown.reset();
            }

            continue;
        }

        let Some(target_entity) = enemy_ai.target else {
            if let Some(mut state) = attack_state {
                state.cooldown.reset();
            }

            continue;
        };

        let Ok((player_transform, mut player_stats, mut health_pool)) =
            player_query.get_mut(target_entity)
        else {
            continue;
        };

        let current_health = health_pool
            .as_deref()
            .map(|pool| pool.current())
            .or_else(|| {
                player_stats
                    .as_deref()
                    .map(|stats| stats.current_health)
            });

        let Some(current_health) = current_health else {
            if let Some(mut state) = attack_state {
                state.cooldown.reset();
            }

            continue;
        };

        if let (Some(pool), Some(stats)) =
            (health_pool.as_deref(), player_stats.as_deref_mut())
        {
            stats.current_health = pool.current();
            stats.max_health = pool.maximum();
        }

        if current_health <= 0.0 {
            if let Some(mut state) = attack_state {
                state.cooldown.reset();
            }

            continue;
        }

        let Some(mut state) = attack_state else {
            commands.entity(enemy_entity).insert(EnemyAttackState {
                target: target_entity,
                cooldown: Timer::from_seconds(
                    DEFAULT_ENEMY_ATTACK_INTERVAL,
                    TimerMode::Repeating,
                ),
            });

            continue;
        };

        if state.target != target_entity {
            state.target = target_entity;
            state.cooldown.reset();
        }

        let distance_squared = enemy_transform
            .translation
            .distance_squared(player_transform.translation);

        if distance_squared > melee_range_squared {
            state.cooldown.reset();
            continue;
        }

        state.cooldown.tick(time.delta());

        if state.cooldown.just_finished() {
            let (current_health, max_health) =
                if let Some(pool) = health_pool.as_deref_mut() {
                    match pool.apply_damage(DEFAULT_ENEMY_ATTACK_DAMAGE) {
                        Ok(_) => {
                            let current_health = pool.current();
                            let max_health = pool.maximum();

                            if let Some(stats) = player_stats.as_deref_mut() {
                                stats.current_health = current_health;
                                stats.max_health = max_health;
                            }

                            (current_health, max_health)
                        }
                        Err(error) => {
                            warn!(
                                "Impossibile applicare il danno alla HealthPool del giocatore: {error}"
                            );
                            continue;
                        }
                    }
                } else if let Some(stats) = player_stats.as_deref_mut() {
                    stats.current_health =
                        (stats.current_health - DEFAULT_ENEMY_ATTACK_DAMAGE)
                            .max(0.0);

                    (stats.current_health, stats.max_health)
                } else {
                    continue;
                };

            info!(
                "Nemico {:?} attacca il giocatore {:?}. Danno: {}. Vita giocatore: {}/{}",
                enemy_entity,
                target_entity,
                DEFAULT_ENEMY_ATTACK_DAMAGE,
                current_health,
                max_health
            );
        }
    }
}

#[cfg(test)]
mod tests;
