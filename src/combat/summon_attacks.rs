use bevy::prelude::*;

use super::{
    AllySummon,
    AllySummonAi,
    CombatStats,
    Enemy,
    EnemyThreat,
    HealthPool,
};
use super::summons::SummonCombatProfile;

#[derive(Component)]
pub(super) struct AllySummonAttackState {
    target: Entity,
    cooldown: Timer,
}

pub(super) fn ally_summon_attack_system(
    time: Res<Time>,
    mut commands: Commands,
    mut summon_query: Query<
        (
            Entity,
            &Transform,
            &AllySummonAi,
            &SummonCombatProfile,
            Option<&mut AllySummonAttackState>,
        ),
        (
            With<AllySummon>,
            Without<Enemy>,
        ),
    >,
    mut enemy_query: Query<
        (
            &Transform,
            Option<&mut CombatStats>,
            Option<&mut HealthPool>,
            Option<&mut EnemyThreat>,
        ),
        (
            With<Enemy>,
            Without<AllySummon>,
        ),
    >,
) {
    for (
        summon_entity,
        summon_transform,
        summon_ai,
        profile,
        attack_state,
    ) in &mut summon_query
    {
        let Some(target_entity) = summon_ai.target else {
            if let Some(mut state) = attack_state {
                state.cooldown.reset();
            }

            continue;
        };

        let attack_damage = profile.attack_damage;
        let attack_interval = profile.attack_interval_seconds;
        let melee_range = profile.melee_range;

        let melee_range_squared = melee_range * melee_range;

        if !attack_damage.is_finite()
            || attack_damage <= 0.0
            || !attack_interval.is_finite()
            || attack_interval <= 0.0
            || !melee_range.is_finite()
            || melee_range < 0.0
            || !melee_range_squared.is_finite()
        {
            if let Some(mut state) = attack_state {
                state.cooldown.reset();
            }

            continue;
        }

        let Ok((
            enemy_transform,
            mut enemy_stats,
            mut enemy_health_pool,
            mut enemy_threat,
        )) = enemy_query.get_mut(target_entity)
        else {
            if let Some(mut state) = attack_state {
                state.cooldown.reset();
            }

            continue;
        };

        let current_health = enemy_health_pool
            .as_deref()
            .map(|pool| pool.current())
            .or_else(|| {
                enemy_stats
                    .as_deref()
                    .map(|stats| stats.current_health)
            });

        let Some(current_health) = current_health else {
            if let Some(mut state) = attack_state {
                state.cooldown.reset();
            }

            continue;
        };

        if let (Some(pool), Some(stats)) = (
            enemy_health_pool.as_deref(),
            enemy_stats.as_deref_mut(),
        ) {
            stats.current_health = pool.current();
            stats.max_health = pool.maximum();
        }

        if !current_health.is_finite() || current_health <= 0.0 {
            if let Some(mut state) = attack_state {
                state.cooldown.reset();
            }

            continue;
        }

        let Some(mut state) = attack_state else {
            commands.entity(summon_entity).insert(
                AllySummonAttackState {
                    target: target_entity,
                    cooldown: Timer::from_seconds(
                        attack_interval,
                        TimerMode::Repeating,
                    ),
                },
            );

            continue;
        };

        if state.target != target_entity {
            state.target = target_entity;
            state.cooldown.reset();
        }

        let distance_squared = summon_transform
            .translation
            .distance_squared(enemy_transform.translation);

        if !distance_squared.is_finite()
            || distance_squared > melee_range_squared
        {
            state.cooldown.reset();
            continue;
        }

        state.cooldown.tick(time.delta());

        if !state.cooldown.just_finished() {
            continue;
        }

        let damage_result =
            if let Some(pool) = enemy_health_pool.as_deref_mut() {
                match pool.apply_damage(attack_damage) {
                    Ok(effective_damage) => {
                        let current_health = pool.current();
                        let maximum_health = pool.maximum();

                        if let Some(stats) = enemy_stats.as_deref_mut() {
                            stats.current_health = current_health;
                            stats.max_health = maximum_health;
                        }

                        Some((
                            effective_damage,
                            current_health,
                            maximum_health,
                        ))
                    }
                    Err(error) => {
                        warn!(
                            "Impossibile applicare il danno dell'evocazione alla HealthPool del nemico: {error}"
                        );

                        None
                    }
                }
            } else if let Some(stats) = enemy_stats.as_deref_mut() {
                let effective_damage =
                    stats.current_health.min(attack_damage).max(0.0);

                stats.current_health =
                    (stats.current_health - attack_damage).max(0.0);

                Some((
                    effective_damage,
                    stats.current_health,
                    stats.max_health,
                ))
            } else {
                None
            };

        let Some((
            effective_damage,
            enemy_current_health,
            enemy_max_health,
        )) = damage_result
        else {
            continue;
        };

        if let Some(threat) = enemy_threat.as_deref_mut() {
            threat.add_threat(summon_entity, effective_damage);
        }

        info!(
            "Evocazione alleata {:?} attacca il nemico {:?}. Danno: {}. Vita: {}/{}",
            summon_entity,
            target_entity,
            effective_damage,
            enemy_current_health,
            enemy_max_health
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::time::Duration;

    fn setup_test_app() -> App {
        let mut app = App::new();

        app.insert_resource(Time::<()>::default())
            .add_systems(Update, ally_summon_attack_system);

        app
    }

    fn advance_test_time(app: &mut App, seconds: f32) {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(seconds));
    }

    fn spawn_enemy(
        world: &mut World,
        position: Vec3,
        health: f32,
        maximum: f32,
    ) -> Entity {
        world.spawn((
            Enemy,
            Transform::from_translation(position),
            CombatStats {
                current_health: health,
                max_health: maximum,
                resource: 0.0,
            },
            HealthPool::new(health, maximum).unwrap(),
        )).id()
    }

    fn spawn_enemy_with_health_pool(
        world: &mut World,
        position: Vec3,
        health: f32,
        maximum: f32,
    ) -> Entity {
        world.spawn((
            Enemy,
            Transform::from_translation(position),
            HealthPool::new(health, maximum).unwrap(),
        )).id()
    }

    fn spawn_ally_summon(
        world: &mut World,
        position: Vec3,
        target: Option<Entity>,
        damage: f32,
        interval: f32,
        melee_range: f32,
    ) -> Entity {
        let mut ai = AllySummonAi::new(15.0);
        ai.target = target;

        world.spawn((
            AllySummon,
            ai,
            SummonCombatProfile {
                attack_damage: damage,
                attack_interval_seconds: interval,
                melee_range,
                aggro_range: 15.0,
                leash_range: 40.0,
                movement_speed: 5.0,
            },
            Transform::from_translation(position),
        )).id()
    }

    #[test]
    fn ally_summon_attacks_enemy_after_configured_cooldown() {
        let mut app = setup_test_app();

        let enemy = spawn_enemy(
            app.world_mut(),
            Vec3::new(0.0, 0.0, 2.0),
            100.0,
            100.0,
        );

        let summon = spawn_ally_summon(
            app.world_mut(),
            Vec3::ZERO,
            Some(enemy),
            12.0,
            2.0,
            2.5,
        );

        app.update();

        advance_test_time(&mut app, 2.0);
        app.update();

        let health_pool = app.world().get::<HealthPool>(enemy).unwrap();
        let stats = app.world().get::<CombatStats>(enemy).unwrap();
        let threat = app.world().get::<EnemyThreat>(enemy).unwrap();

        assert_eq!(health_pool.current(), 88.0);
        assert_eq!(stats.current_health, 88.0);
        assert_eq!(
            threat.values.get(&summon),
            Some(&12.0)
        );
    }

    #[test]
    fn ally_summon_uses_health_pool_without_combat_stats() {
        let mut app = setup_test_app();

        let enemy = spawn_enemy_with_health_pool(
            app.world_mut(),
            Vec3::new(0.0, 0.0, 2.0),
            100.0,
            150.0,
        );

        let summon = spawn_ally_summon(
            app.world_mut(),
            Vec3::ZERO,
            Some(enemy),
            15.0,
            1.0,
            2.5,
        );

        app.update();

        advance_test_time(&mut app, 1.0);
        app.update();

        let health_pool = app.world().get::<HealthPool>(enemy).unwrap();
        let threat = app.world().get::<EnemyThreat>(enemy).unwrap();

        assert_eq!(health_pool.current(), 85.0);
        assert_eq!(health_pool.maximum(), 150.0);
        assert!(app.world().get::<CombatStats>(enemy).is_none());
        assert_eq!(
            threat.values.get(&summon),
            Some(&15.0)
        );
    }

    #[test]
    fn ally_summon_damage_cannot_exceed_remaining_health() {
        let mut app = setup_test_app();

        let enemy = spawn_enemy(
            app.world_mut(),
            Vec3::new(0.0, 0.0, 2.0),
            5.0,
            100.0,
        );

        let summon = spawn_ally_summon(
            app.world_mut(),
            Vec3::ZERO,
            Some(enemy),
            20.0,
            1.0,
            2.5,
        );

        app.update();

        advance_test_time(&mut app, 1.0);
        app.update();

        let health_pool = app.world().get::<HealthPool>(enemy).unwrap();
        let stats = app.world().get::<CombatStats>(enemy).unwrap();
        let threat = app.world().get::<EnemyThreat>(enemy).unwrap();

        assert_eq!(health_pool.current(), 0.0);
        assert_eq!(stats.current_health, 0.0);
        assert_eq!(
            threat.values.get(&summon),
            Some(&5.0)
        );
    }

    #[test]
    fn ally_summon_does_not_attack_outside_melee_range() {
        let mut app = setup_test_app();

        let enemy = spawn_enemy(
            app.world_mut(),
            Vec3::new(0.0, 0.0, 5.0),
            100.0,
            100.0,
        );

        let summon = spawn_ally_summon(
            app.world_mut(),
            Vec3::ZERO,
            Some(enemy),
            10.0,
            2.0,
            2.5,
        );

        app.update();

        advance_test_time(&mut app, 2.0);
        app.update();

        assert_eq!(
            app.world().get::<HealthPool>(enemy).unwrap().current(),
            100.0
        );

        assert_eq!(
            app.world()
                .get::<AllySummonAttackState>(summon)
                .unwrap()
                .cooldown
                .elapsed(),
            Duration::ZERO
        );

        app.world_mut()
            .get_mut::<Transform>(enemy)
            .unwrap()
            .translation = Vec3::new(0.0, 0.0, 1.0);

        advance_test_time(&mut app, 1.9);
        app.update();

        assert_eq!(
            app.world().get::<HealthPool>(enemy).unwrap().current(),
            100.0
        );

        advance_test_time(&mut app, 0.2);
        app.update();

        assert_eq!(
            app.world().get::<HealthPool>(enemy).unwrap().current(),
            90.0
        );
    }

    #[test]
    fn ally_summon_resets_cooldown_when_target_changes() {
        let mut app = setup_test_app();

        let first_enemy = spawn_enemy(
            app.world_mut(),
            Vec3::new(0.0, 0.0, 1.0),
            100.0,
            100.0,
        );

        let second_enemy = spawn_enemy(
            app.world_mut(),
            Vec3::new(0.0, 0.0, 2.0),
            100.0,
            100.0,
        );

        let summon = spawn_ally_summon(
            app.world_mut(),
            Vec3::ZERO,
            Some(first_enemy),
            10.0,
            2.0,
            2.5,
        );

        app.update();

        advance_test_time(&mut app, 1.0);
        app.update();

        app.world_mut()
            .get_mut::<AllySummonAi>(summon)
            .unwrap()
            .target = Some(second_enemy);

        advance_test_time(&mut app, 0.1);
        app.update();

        assert_eq!(
            app.world().get::<HealthPool>(first_enemy).unwrap().current(),
            100.0
        );

        assert_eq!(
            app.world().get::<HealthPool>(second_enemy).unwrap().current(),
            100.0
        );

        advance_test_time(&mut app, 1.8);
        app.update();

        assert_eq!(
            app.world().get::<HealthPool>(second_enemy).unwrap().current(),
            100.0
        );

        advance_test_time(&mut app, 0.2);
        app.update();

        assert_eq!(
            app.world().get::<HealthPool>(second_enemy).unwrap().current(),
            90.0
        );
    }

    #[test]
    fn ally_summon_does_not_attack_defeated_enemy() {
        let mut app = setup_test_app();

        let enemy = spawn_enemy(
            app.world_mut(),
            Vec3::new(0.0, 0.0, 1.0),
            0.0,
            100.0,
        );

        let summon = spawn_ally_summon(
            app.world_mut(),
            Vec3::ZERO,
            Some(enemy),
            10.0,
            1.0,
            2.5,
        );

        app.update();

        advance_test_time(&mut app, 1.0);
        app.update();

        let health_pool = app.world().get::<HealthPool>(enemy).unwrap();
        let stats = app.world().get::<CombatStats>(enemy).unwrap();
        let attack_state =
            app.world().get::<AllySummonAttackState>(summon).unwrap();

        assert_eq!(health_pool.current(), 0.0);
        assert_eq!(stats.current_health, 0.0);
        assert_eq!(attack_state.cooldown.elapsed(), Duration::ZERO);
    }

    #[test]
    fn ally_summon_without_target_does_not_attack() {
        let mut app = setup_test_app();

        let enemy = spawn_enemy(
            app.world_mut(),
            Vec3::new(0.0, 0.0, 1.0),
            100.0,
            100.0,
        );

        spawn_ally_summon(
            app.world_mut(),
            Vec3::ZERO,
            None,
            10.0,
            1.0,
            2.5,
        );

        app.update();

        advance_test_time(&mut app, 2.0);
        app.update();

        assert_eq!(
            app.world().get::<HealthPool>(enemy).unwrap().current(),
            100.0
        );
    }

    #[test]
    fn ally_summon_rejects_invalid_attack_profile() {
        let mut app = setup_test_app();

        let enemy = spawn_enemy(
            app.world_mut(),
            Vec3::new(0.0, 0.0, 1.0),
            100.0,
            100.0,
        );

        let summon = spawn_ally_summon(
            app.world_mut(),
            Vec3::ZERO,
            Some(enemy),
            10.0,
            0.0,
            2.5,
        );

        app.update();

        advance_test_time(&mut app, 2.0);
        app.update();

        assert_eq!(
            app.world().get::<HealthPool>(enemy).unwrap().current(),
            100.0
        );

        assert!(
            app.world()
                .get::<AllySummonAttackState>(summon)
                .is_none()
        );
    }
}
