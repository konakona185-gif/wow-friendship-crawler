
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
        (&Transform, &mut CombatStats, Option<&mut HealthPool>),
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

        let Ok((player_transform, mut player_stats, health_pool)) =
            player_query.get_mut(target_entity)
        else {
            continue;
        };

        if let Some(pool) = health_pool.as_deref() {
            player_stats.current_health = pool.current();
            player_stats.max_health = pool.maximum();
        }

        if player_stats.current_health <= 0.0 {
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
            if let Some(mut health_pool) = health_pool {
                match health_pool.apply_damage(DEFAULT_ENEMY_ATTACK_DAMAGE) {
                    Ok(_) => {
                        player_stats.current_health = health_pool.current();
                    }
                    Err(error) => {
                        warn!(
                            "Impossibile applicare il danno alla HealthPool del giocatore: {error}"
                        );
                        continue;
                    }
                }
            } else {
                player_stats.current_health =
                    (player_stats.current_health - DEFAULT_ENEMY_ATTACK_DAMAGE)
                        .max(0.0);
            }

            info!(
                "Nemico {:?} attacca il giocatore {:?}. Danno: {}. Vita giocatore: {}/{}",
                enemy_entity,
                target_entity,
                DEFAULT_ENEMY_ATTACK_DAMAGE,
                player_stats.current_health,
                player_stats.max_health
            );
        }
    }
}

#[cfg(test)]
mod tests;
