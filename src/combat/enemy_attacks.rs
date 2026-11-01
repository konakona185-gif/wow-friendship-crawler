use bevy::prelude::*;

use super::*;

#[derive(Component)]
struct EnemyAttackState {
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
        (&Transform, &mut CombatStats),
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

        let Ok((player_transform, mut player_stats)) =
            player_query.get_mut(target_entity)
        else {
            continue;
        };

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
            player_stats.current_health =
                (player_stats.current_health - DEFAULT_ENEMY_ATTACK_DAMAGE)
                    .max(0.0);

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
mod tests {
    use super::*;
    use std::time::Duration;

    fn setup_enemy_attack_test_app() -> App {
        let mut app = App::new();

        app.insert_resource(Time::<()>::default());
        app.add_systems(Update, enemy_attack_system);

        app
    }

    fn advance_test_time(app: &mut App, seconds: f32) {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(seconds));
    }

    fn test_enemy_attack_state(target: Entity) -> EnemyAttackState {
        EnemyAttackState {
            target,
            cooldown: Timer::from_seconds(
                DEFAULT_ENEMY_ATTACK_INTERVAL,
                TimerMode::Repeating,
            ),
        }
    }

    fn spawn_living_player(
        world: &mut World,
        position: Vec3,
    ) -> Entity {
        world.spawn((
            Character,
            Transform::from_translation(position),
            CombatStats {
                current_health: 100.0,
                max_health: 100.0,
                resource: 0.0,
            },
        )).id()
    }

    #[test]
    fn enemy_attacks_a_living_player_after_the_melee_cooldown() {
        let mut app = setup_enemy_attack_test_app();

        let player = spawn_living_player(
            app.world_mut(),
            Vec3::new(0.0, 0.0, 1.0),
        );

        app.world_mut().spawn((
            Enemy,
            EnemyAi {
                target: Some(player),
                ..EnemyAi::default()
            },
            EnemyAttackState {
                target: player,
                cooldown: Timer::from_seconds(
                    DEFAULT_ENEMY_ATTACK_INTERVAL,
                    TimerMode::Repeating,
                ),
            },
            Transform::default(),
        ));

        advance_test_time(&mut app, DEFAULT_ENEMY_ATTACK_INTERVAL);
        app.update();

        assert_eq!(
            app.world().get::<CombatStats>(player).unwrap().current_health,
            100.0 - DEFAULT_ENEMY_ATTACK_DAMAGE
        );
    }

    #[test]
    fn enemy_does_not_attack_outside_melee_range() {
        let mut app = setup_enemy_attack_test_app();

        let player = spawn_living_player(
            app.world_mut(),
            Vec3::new(0.0, 0.0, DEFAULT_ENEMY_MELEE_RANGE + 1.0),
        );

        app.world_mut().spawn((
            Enemy,
            EnemyAi {
                target: Some(player),
                ..EnemyAi::default()
            },
            test_enemy_attack_state(player),
            Transform::default(),
        ));

        advance_test_time(&mut app, DEFAULT_ENEMY_ATTACK_INTERVAL);
        app.update();

        assert_eq!(
            app.world().get::<CombatStats>(player).unwrap().current_health,
            100.0
        );

        app.world_mut()
            .get_mut::<Transform>(player)
            .unwrap()
            .translation = Vec3::new(0.0, 0.0, 1.0);

        advance_test_time(&mut app, DEFAULT_ENEMY_ATTACK_INTERVAL - 0.1);
        app.update();

        assert_eq!(
            app.world().get::<CombatStats>(player).unwrap().current_health,
            100.0
        );

        advance_test_time(&mut app, 0.1);
        app.update();

        assert_eq!(
            app.world().get::<CombatStats>(player).unwrap().current_health,
            100.0 - DEFAULT_ENEMY_ATTACK_DAMAGE
        );
    }

    #[test]
    fn returning_enemies_do_not_attack_players() {
        let mut app = setup_enemy_attack_test_app();

        let player = spawn_living_player(
            app.world_mut(),
            Vec3::new(0.0, 0.0, 1.0),
        );

        app.world_mut().spawn((
            Enemy,
            EnemyAi {
                target: Some(player),
                returning_home: true,
                ..EnemyAi::default()
            },
            test_enemy_attack_state(player),
            Transform::default(),
        ));

        advance_test_time(&mut app, DEFAULT_ENEMY_ATTACK_INTERVAL);
        app.update();

        assert_eq!(
            app.world().get::<CombatStats>(player).unwrap().current_health,
            100.0
        );
    }

    #[test]
    fn defeated_players_cannot_be_damaged_by_enemy_attacks() {
        let mut app = setup_enemy_attack_test_app();

        let player = app.world_mut().spawn((
            Character,
            Transform::from_xyz(0.0, 0.0, 1.0),
            CombatStats {
                current_health: 0.0,
                max_health: 100.0,
                resource: 0.0,
            },
        )).id();

        app.world_mut().spawn((
            Enemy,
            EnemyAi {
                target: Some(player),
                ..EnemyAi::default()
            },
            test_enemy_attack_state(player),
            Transform::default(),
        ));

        advance_test_time(&mut app, DEFAULT_ENEMY_ATTACK_INTERVAL);
        app.update();

        assert_eq!(
            app.world().get::<CombatStats>(player).unwrap().current_health,
            0.0
        );
    }
}
