use bevy::prelude::*;

use super::*;

pub(super) fn player_cast_system(
    mut player_query: Query<
        (Entity, &FellowshipRole, &mut CombatStats, &Target, &mut CharacterCommand),
        With<Character>,
    >,
    mut enemy_query: Query<
        (&mut CombatStats, Option<&EnemyAi>, Option<&mut EnemyThreat>),
        (With<Enemy>, Without<Character>),
    >,
) {
    for (player_entity, role, mut player_stats, target, mut command) in &mut player_query {
        if !std::mem::take(&mut command.ability_1) {
            continue;
        }

        if player_stats.current_health <= 0.0 {
            continue;
        }

        let Some(target_entity) = target.entity else {
            info!("Nessun target selezionato! Premi TAB per agganciare il nemico.");
            continue;
        };

        let Ok((mut enemy_stats, enemy_ai, enemy_threat)) =
            enemy_query.get_mut(target_entity)
        else {
            continue;
        };

        if enemy_stats.current_health <= 0.0
            || enemy_ai.is_some_and(|ai| ai.returning_home)
        {
            continue;
        }

        let damage = match role {
            FellowshipRole::Tank => {
                let damage = enemy_stats.current_health.min(15.0).max(0.0);
                enemy_stats.current_health =
                    (enemy_stats.current_health - 15.0).max(0.0);
                player_stats.resource += 20.0;

                info!(
                    "Tank colpisce! Danno: {}. Vita Nemico: {}/{}. Rabbia generata: {}",
                    damage,
                    enemy_stats.current_health,
                    enemy_stats.max_health,
                    player_stats.resource
                );

                damage
            }
            FellowshipRole::Dps => {
                if player_stats.resource >= 30.0 {
                    player_stats.resource -= 30.0;
                    let damage = enemy_stats.current_health.min(40.0).max(0.0);
                    enemy_stats.current_health =
                        (enemy_stats.current_health - 40.0).max(0.0);

                    info!(
                        "DPS scatta l'abilità! Danno pesante: {}. Vita Nemico: {}/{}. Energia residua: {}",
                        damage,
                        enemy_stats.current_health,
                        enemy_stats.max_health,
                        player_stats.resource
                    );

                    damage
                } else {
                    let damage = enemy_stats.current_health.min(10.0).max(0.0);
                    enemy_stats.current_health =
                        (enemy_stats.current_health - 10.0).max(0.0);
                    player_stats.resource += 15.0;

                    info!(
                        "DPS attacco base! Danno: {}. Vita Nemico: {}/{}. Energia rigenerata: {}",
                        damage,
                        enemy_stats.current_health,
                        enemy_stats.max_health,
                        player_stats.resource
                    );

                    damage
                }
            }
            FellowshipRole::Healer => {
                if player_stats.resource >= 20.0 {
                    player_stats.resource -= 20.0;
                    let damage = enemy_stats.current_health.min(25.0).max(0.0);
                    enemy_stats.current_health =
                        (enemy_stats.current_health - 25.0).max(0.0);

                    info!(
                        "Healer lancia incantesimo! Danno: {}. Vita Nemico: {}/{}. Mana residuo: {}",
                        damage,
                        enemy_stats.current_health,
                        enemy_stats.max_health,
                        player_stats.resource
                    );

                    damage
                } else {
                    info!("Mana insufficiente per curare/attaccare!");
                    0.0
                }
            }
        };

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
mod tests {
    use super::*;

    #[test]
    fn defeated_targets_cannot_receive_additional_player_casts() {
        let mut app = App::new();
        app.add_systems(Update, player_cast_system);

        let enemy = app.world_mut().spawn((
            Enemy,
            CombatStats {
                current_health: 0.0,
                max_health: 100.0,
                resource: 0.0,
            },
        )).id();

        let player = app.world_mut().spawn((
            Character,
            FellowshipRole::Dps,
            CombatStats {
                current_health: 100.0,
                max_health: 100.0,
                resource: 100.0,
            },
            Target {
                entity: Some(enemy),
            },
            CharacterCommand {
                ability_1: true,
                ..default()
            },
        )).id();

        app.update();

        assert_eq!(
            app.world().get::<CombatStats>(enemy).unwrap().current_health,
            0.0
        );

        assert_eq!(
            app.world().get::<CombatStats>(player).unwrap().resource,
            100.0
        );
    }

    #[test]
    fn returning_enemies_are_immune_to_player_abilities() {
        let mut app = App::new();
        app.add_systems(Update, player_cast_system);

        let enemy = app.world_mut().spawn((
            Enemy,
            EnemyAi {
                target: None,
                aggro_range: DEFAULT_ENEMY_AGGRO_RANGE,
                leash_range: DEFAULT_ENEMY_LEASH_RANGE,
                returning_home: true,
            },
            EnemyHomePosition(Vec3::ZERO),
            Transform::from_xyz(41.0, 0.0, 0.0),
            CombatStats {
                current_health: 100.0,
                max_health: 100.0,
                resource: 0.0,
            },
        )).id();

        let player = app.world_mut().spawn((
            Character,
            FellowshipRole::Dps,
            CombatStats {
                current_health: 100.0,
                max_health: 100.0,
                resource: 100.0,
            },
            Target {
                entity: Some(enemy),
            },
            CharacterCommand {
                ability_1: true,
                ..default()
            },
        )).id();

        app.update();

        assert_eq!(
            app.world().get::<CombatStats>(enemy).unwrap().current_health,
            100.0
        );

        assert_eq!(
            app.world().get::<CombatStats>(player).unwrap().resource,
            100.0
        );
    }

    #[test]
    fn defeated_players_cannot_cast_abilities() {
        let mut app = App::new();
        app.add_systems(Update, player_cast_system);

        let enemy = app.world_mut().spawn((
            Enemy,
            CombatStats {
                current_health: 100.0,
                max_health: 100.0,
                resource: 0.0,
            },
        )).id();

        let player = app.world_mut().spawn((
            Character,
            FellowshipRole::Tank,
            CombatStats {
                current_health: 0.0,
                max_health: 100.0,
                resource: 0.0,
            },
            Target {
                entity: Some(enemy),
            },
            CharacterCommand {
                ability_1: true,
                ..default()
            },
        )).id();

        app.update();

        assert_eq!(
            app.world().get::<CombatStats>(enemy).unwrap().current_health,
            100.0
        );

        assert_eq!(
            app.world().get::<CombatStats>(player).unwrap().resource,
            0.0
        );
    }

    #[test]
    fn player_damage_never_reduces_enemy_health_below_zero() {
        let mut app = App::new();
        app.add_systems(Update, player_cast_system);

        let enemy = app.world_mut().spawn((
            Enemy,
            CombatStats {
                current_health: 5.0,
                max_health: 100.0,
                resource: 0.0,
            },
        )).id();

        let player = app.world_mut().spawn((
            Character,
            FellowshipRole::Tank,
            CombatStats {
                current_health: 100.0,
                max_health: 100.0,
                resource: 0.0,
            },
            Target {
                entity: Some(enemy),
            },
            CharacterCommand {
                ability_1: true,
                ..default()
            },
        )).id();

        app.update();

        assert_eq!(
            app.world().get::<CombatStats>(enemy).unwrap().current_health,
            0.0
        );

        assert_eq!(
            app.world().get::<CombatStats>(player).unwrap().resource,
            20.0
        );
    }

    #[test]
    fn tank_damage_generates_weighted_threat() {
        let mut app = App::new();
        app.add_systems(Update, player_cast_system);

        let enemy = app.world_mut().spawn((
            Enemy,
            EnemyAi::default(),
            EnemyThreat::default(),
            CombatStats {
                current_health: 100.0,
                max_health: 100.0,
                resource: 0.0,
            },
        )).id();

        let player = app.world_mut().spawn((
            Character,
            FellowshipRole::Tank,
            CombatStats {
                current_health: 100.0,
                max_health: 100.0,
                resource: 0.0,
            },
            Target {
                entity: Some(enemy),
            },
            CharacterCommand {
                ability_1: true,
                ..default()
            },
        )).id();

        app.update();

        let threat = app.world().get::<EnemyThreat>(enemy).unwrap();

        assert_eq!(
            threat.values.get(&player),
            Some(&(15.0 * DEFAULT_TANK_THREAT_MULTIPLIER))
        );
    }
}
