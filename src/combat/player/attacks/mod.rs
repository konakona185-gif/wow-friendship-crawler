use bevy::prelude::*;

use crate::combat::{
    CombatStats, Enemy, EnemyAi, EnemyThreat, FellowshipRole, HealthPool, Target,
    DEFAULT_TANK_THREAT_MULTIPLIER,
};
use crate::player::{Character, CharacterCommand};

#[derive(Clone, Copy)]
enum PlayerAttackKind {
    TankStrike,
    DpsSpecial,
    DpsBasic,
    HealerSpell,
    HealerInsufficientMana,
}

pub(super) fn player_cast_system(
    mut player_query: Query<
        (
            Entity,
            &FellowshipRole,
            &mut CombatStats,
            &Target,
            &mut CharacterCommand,
            Option<&HealthPool>,
        ),
        With<Character>,
    >,
    mut enemy_query: Query<
        (
            Option<&mut CombatStats>,
            Option<&EnemyAi>,
            Option<&mut EnemyThreat>,
            Option<&mut HealthPool>,
        ),
        (With<Enemy>, Without<Character>),
    >,
) {
    for (player_entity, role, mut player_stats, target, mut command, player_health_pool) in
        &mut player_query
    {
        if !std::mem::take(&mut command.ability_1) {
            continue;
        }

        if let Some(pool) = player_health_pool {
            player_stats.current_health = pool.current();
            player_stats.max_health = pool.maximum();
        }

        if player_stats.current_health <= 0.0 {
            continue;
        }

        let Some(target_entity) = target.entity else {
            info!("Nessun target selezionato! Premi TAB per agganciare il nemico.");
            continue;
        };

        let Ok((mut enemy_stats, enemy_ai, enemy_threat, enemy_health_pool)) =
            enemy_query.get_mut(target_entity)
        else {
            continue;
        };

        let current_enemy_health = enemy_health_pool
            .as_deref()
            .map(|pool| pool.current())
            .or_else(|| enemy_stats.as_deref().map(|stats| stats.current_health));

        let Some(current_enemy_health) = current_enemy_health else {
            continue;
        };

        if let (Some(pool), Some(stats)) =
            (enemy_health_pool.as_deref(), enemy_stats.as_deref_mut())
        {
            stats.current_health = pool.current();
            stats.max_health = pool.maximum();
        }

        if current_enemy_health <= 0.0 || enemy_ai.is_some_and(|ai| ai.returning_home) {
            continue;
        }

        let (requested_damage, attack_kind) = match role {
            FellowshipRole::Tank => {
                player_stats.resource += 20.0;

                (15.0, PlayerAttackKind::TankStrike)
            }
            FellowshipRole::Dps => {
                if player_stats.resource >= 30.0 {
                    player_stats.resource -= 30.0;

                    (40.0, PlayerAttackKind::DpsSpecial)
                } else {
                    player_stats.resource += 15.0;

                    (10.0, PlayerAttackKind::DpsBasic)
                }
            }
            FellowshipRole::Healer => {
                if player_stats.resource >= 20.0 {
                    player_stats.resource -= 20.0;

                    (25.0, PlayerAttackKind::HealerSpell)
                } else {
                    (0.0, PlayerAttackKind::HealerInsufficientMana)
                }
            }
        };

        if matches!(attack_kind, PlayerAttackKind::HealerInsufficientMana) {
            info!("Mana insufficiente per curare/attaccare!");
            continue;
        }

        let (damage, enemy_current_health, enemy_max_health) =
            if let Some(mut health_pool) = enemy_health_pool {
                match health_pool.apply_damage(requested_damage) {
                    Ok(effective_damage) => {
                        let current_health = health_pool.current();
                        let maximum_health = health_pool.maximum();

                        if let Some(stats) = enemy_stats.as_deref_mut() {
                            stats.current_health = current_health;
                            stats.max_health = maximum_health;
                        }

                        (effective_damage, current_health, maximum_health)
                    }
                    Err(error) => {
                        warn!("Impossibile applicare il danno alla HealthPool del nemico: {error}");
                        continue;
                    }
                }
            } else if let Some(stats) = enemy_stats.as_deref_mut() {
                let effective_damage = stats.current_health.min(requested_damage).max(0.0);

                stats.current_health = (stats.current_health - requested_damage).max(0.0);

                (effective_damage, stats.current_health, stats.max_health)
            } else {
                continue;
            };

        match attack_kind {
            PlayerAttackKind::TankStrike => {
                info!(
                    "Tank colpisce! Danno: {}. Vita Nemico: {}/{}. Rabbia generata: {}",
                    damage, enemy_current_health, enemy_max_health, player_stats.resource
                );
            }
            PlayerAttackKind::DpsSpecial => {
                info!(
                    "DPS scatta l'abilità! Danno pesante: {}. Vita Nemico: {}/{}. Energia residua: {}",
                    damage,
                    enemy_current_health,
                    enemy_max_health,
                    player_stats.resource
                );
            }
            PlayerAttackKind::DpsBasic => {
                info!(
                    "DPS attacco base! Danno: {}. Vita Nemico: {}/{}. Energia rigenerata: {}",
                    damage, enemy_current_health, enemy_max_health, player_stats.resource
                );
            }
            PlayerAttackKind::HealerSpell => {
                info!(
                    "Healer lancia incantesimo! Danno: {}. Vita Nemico: {}/{}. Mana residuo: {}",
                    damage, enemy_current_health, enemy_max_health, player_stats.resource
                );
            }
            PlayerAttackKind::HealerInsufficientMana => {}
        }

        if let Some(mut threat) = enemy_threat {
            let multiplier = match role {
                FellowshipRole::Tank => DEFAULT_TANK_THREAT_MULTIPLIER,
                FellowshipRole::Healer | FellowshipRole::Dps => 1.0,
            };

            threat.add_threat(player_entity, damage * multiplier);
        }
    }
}

#[cfg(test)]
mod tests;
