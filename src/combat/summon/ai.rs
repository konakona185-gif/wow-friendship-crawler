use bevy::prelude::*;

use crate::combat::{
    AllySummon,
    CombatStats,
    Enemy,
    HealthPool,
    SummonedCreature,
    SummonFaction,
};
use super::{
    AllySummonAi,
    SummonCombatProfile,
};

use crate::content::ContentId;

const DEFAULT_SUMMON_AGGRO_RANGE: f32 = 15.0;
const DEFAULT_SUMMON_LEASH_RANGE: f32 = 40.0;

fn combatant_is_alive(
    stats: Option<&CombatStats>,
    health_pool: Option<&HealthPool>,
) -> bool {
    health_pool
        .map(|pool| pool.current())
        .or_else(|| stats.map(|stats| stats.current_health))
        .is_some_and(|health| health.is_finite() && health > 0.0)
}

fn valid_aggro_range_squared(aggro_range: f32) -> f32 {
    let range = if aggro_range.is_finite() && aggro_range >= 0.0 {
        aggro_range
    } else {
        DEFAULT_SUMMON_AGGRO_RANGE
    };

    let range_squared = range * range;

    if range_squared.is_finite() {
        range_squared
    } else {
        DEFAULT_SUMMON_AGGRO_RANGE * DEFAULT_SUMMON_AGGRO_RANGE
    }
}

fn valid_leash_range_squared(leash_range: f32) -> f32 {
    let range = if leash_range.is_finite() && leash_range > 0.0 {
        leash_range
    } else {
        DEFAULT_SUMMON_LEASH_RANGE
    };

    let range_squared = range * range;

    if range_squared.is_finite() {
        range_squared
    } else {
        DEFAULT_SUMMON_LEASH_RANGE * DEFAULT_SUMMON_LEASH_RANGE
    }
}

pub(super) fn ally_summon_aggro_system(
    mut summon_query: Query<
        (
            &Transform,
            &SummonedCreature,
            &SummonCombatProfile,
            &mut AllySummonAi,
        ),
        (
            With<AllySummon>,
            Without<Enemy>,
        ),
    >,
    enemy_query: Query<
        (
            Entity,
            &Transform,
            Option<&CombatStats>,
            Option<&HealthPool>,
        ),
        (
            With<Enemy>,
            Without<AllySummon>,
        ),
    >,
    owner_query: Query<&Transform>,
) {
    for (
        summon_transform,
        summoned_creature,
        combat_profile,
        mut summon_ai,
    ) in &mut summon_query
    {
        let leash_range_squared =
            valid_leash_range_squared(combat_profile.leash_range);

        let owner_is_within_leash = owner_query
            .get(summoned_creature.owner)
            .is_ok_and(|owner_transform| {
                let distance_squared = summon_transform
                    .translation
                    .distance_squared(owner_transform.translation);

                distance_squared.is_finite()
                    && distance_squared <= leash_range_squared
            });

        if !owner_is_within_leash {
            summon_ai.target = None;
            continue;
        }

        let aggro_range_squared =
            valid_aggro_range_squared(summon_ai.aggro_range);

        let current_target_is_valid = summon_ai.target.is_some_and(
            |target_entity| {
                enemy_query
                    .get(target_entity)
                    .is_ok_and(
                        |(_, enemy_transform, stats, health_pool)| {
                            if !combatant_is_alive(stats, health_pool) {
                                return false;
                            }

                            let distance_squared = summon_transform
                                .translation
                                .distance_squared(
                                    enemy_transform.translation,
                                );

                            distance_squared.is_finite()
                                && distance_squared <= aggro_range_squared
                        },
                    )
            },
        );

        if current_target_is_valid {
            continue;
        }

        let new_target = enemy_query
            .iter()
            .filter_map(
                |(entity, enemy_transform, stats, health_pool)| {
                    if !combatant_is_alive(stats, health_pool) {
                        return None;
                    }

                    let distance_squared = summon_transform
                        .translation
                        .distance_squared(
                            enemy_transform.translation,
                        );

                    (distance_squared.is_finite()
                        && distance_squared <= aggro_range_squared)
                        .then_some((entity, distance_squared))
                },
            )
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(entity, _)| entity);

        summon_ai.target = new_target;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_app() -> App {
        let mut app = App::new();
        app.add_systems(Update, ally_summon_aggro_system);
        app
    }

    fn spawn_enemy(
        world: &mut World,
        position: Vec3,
        health: f32,
    ) -> Entity {
        world.spawn((
            Enemy,
            Transform::from_translation(position),
            CombatStats {
                current_health: health,
                max_health: 100.0,
                resource: 0.0,
            },
        )).id()
    }

    fn spawn_enemy_with_health_pool(
        world: &mut World,
        position: Vec3,
        stats_health: f32,
        pool_health: f32,
    ) -> Entity {
        world.spawn((
            Enemy,
            Transform::from_translation(position),
            CombatStats {
                current_health: stats_health,
                max_health: 100.0,
                resource: 0.0,
            },
            HealthPool::new(pool_health, 100.0).unwrap(),
        )).id()
    }

    fn spawn_ally_summon(
        world: &mut World,
        position: Vec3,
        aggro_range: f32,
    ) -> Entity {
        let owner = world.spawn(
            Transform::from_translation(position),
        ).id();

        world.spawn((
            AllySummon,
            AllySummonAi::new(aggro_range),
            SummonedCreature::new(
                owner,
                ContentId::new("test:creature.imp").unwrap(),
                SummonFaction::Allies,
                None,
            ),
            SummonCombatProfile {
                attack_damage: 12.0,
                attack_interval_seconds: 2.0,
                melee_range: 2.5,
                aggro_range,
                leash_range: 40.0,
                movement_speed: 5.0,
            },
            Transform::from_translation(position),
        )).id()
    }

    #[test]
    fn ally_summon_acquires_the_nearest_enemy() {
        let mut app = setup_test_app();

        let summon = spawn_ally_summon(
            app.world_mut(),
            Vec3::ZERO,
            15.0,
        );

        spawn_enemy(
            app.world_mut(),
            Vec3::new(0.0, 0.0, 12.0),
            100.0,
        );

        let nearest_enemy = spawn_enemy(
            app.world_mut(),
            Vec3::new(0.0, 0.0, 5.0),
            100.0,
        );

        app.update();

        assert_eq!(
            app.world().get::<AllySummonAi>(summon).unwrap().target,
            Some(nearest_enemy)
        );
    }

    #[test]
    fn ally_summon_ignores_enemies_outside_aggro_range() {
        let mut app = setup_test_app();

        let summon = spawn_ally_summon(
            app.world_mut(),
            Vec3::ZERO,
            10.0,
        );

        spawn_enemy(
            app.world_mut(),
            Vec3::new(0.0, 0.0, 11.0),
            100.0,
        );

        app.update();

        assert_eq!(
            app.world().get::<AllySummonAi>(summon).unwrap().target,
            None
        );
    }

    #[test]
    fn ally_summon_ignores_defeated_enemies() {
        let mut app = setup_test_app();

        let summon = spawn_ally_summon(
            app.world_mut(),
            Vec3::ZERO,
            15.0,
        );

        spawn_enemy(
            app.world_mut(),
            Vec3::new(0.0, 0.0, 2.0),
            0.0,
        );

        let living_enemy = spawn_enemy(
            app.world_mut(),
            Vec3::new(0.0, 0.0, 8.0),
            100.0,
        );

        app.update();

        assert_eq!(
            app.world().get::<AllySummonAi>(summon).unwrap().target,
            Some(living_enemy)
        );
    }

    #[test]
    fn ally_summon_uses_health_pool_to_determine_enemy_liveness() {
        let mut app = setup_test_app();

        let summon = spawn_ally_summon(
            app.world_mut(),
            Vec3::ZERO,
            15.0,
        );

        spawn_enemy_with_health_pool(
            app.world_mut(),
            Vec3::new(0.0, 0.0, 2.0),
            100.0,
            0.0,
        );

        let living_enemy = spawn_enemy_with_health_pool(
            app.world_mut(),
            Vec3::new(0.0, 0.0, 8.0),
            0.0,
            50.0,
        );

        app.update();

        assert_eq!(
            app.world().get::<AllySummonAi>(summon).unwrap().target,
            Some(living_enemy)
        );
    }

    #[test]
    fn ally_summon_keeps_its_current_target_while_valid() {
        let mut app = setup_test_app();

        let summon = spawn_ally_summon(
            app.world_mut(),
            Vec3::ZERO,
            15.0,
        );

        let first_enemy = spawn_enemy(
            app.world_mut(),
            Vec3::new(0.0, 0.0, 10.0),
            100.0,
        );

        app.update();

        assert_eq!(
            app.world().get::<AllySummonAi>(summon).unwrap().target,
            Some(first_enemy)
        );

        let closer_enemy = spawn_enemy(
            app.world_mut(),
            Vec3::new(0.0, 0.0, 2.0),
            100.0,
        );

        app.update();

        assert_eq!(
            app.world().get::<AllySummonAi>(summon).unwrap().target,
            Some(first_enemy)
        );

        assert_ne!(
            app.world().get::<AllySummonAi>(summon).unwrap().target,
            Some(closer_enemy)
        );
    }

    #[test]
    fn ally_summon_reacquires_when_its_current_target_dies() {
        let mut app = setup_test_app();

        let summon = spawn_ally_summon(
            app.world_mut(),
            Vec3::ZERO,
            15.0,
        );

        let first_enemy = spawn_enemy(
            app.world_mut(),
            Vec3::new(0.0, 0.0, 5.0),
            100.0,
        );

        let second_enemy = spawn_enemy(
            app.world_mut(),
            Vec3::new(0.0, 0.0, 9.0),
            100.0,
        );

        app.update();

        assert_eq!(
            app.world().get::<AllySummonAi>(summon).unwrap().target,
            Some(first_enemy)
        );

        app.world_mut()
            .get_mut::<CombatStats>(first_enemy)
            .unwrap()
            .current_health = 0.0;

        app.update();

        assert_eq!(
            app.world().get::<AllySummonAi>(summon).unwrap().target,
            Some(second_enemy)
        );
    }

    #[test]
    fn ally_summon_drops_target_when_it_moves_out_of_range() {
        let mut app = setup_test_app();

        let summon = spawn_ally_summon(
            app.world_mut(),
            Vec3::ZERO,
            10.0,
        );

        let enemy = spawn_enemy(
            app.world_mut(),
            Vec3::new(0.0, 0.0, 5.0),
            100.0,
        );

        app.update();

        assert_eq!(
            app.world().get::<AllySummonAi>(summon).unwrap().target,
            Some(enemy)
        );

        app.world_mut()
            .get_mut::<Transform>(enemy)
            .unwrap()
            .translation = Vec3::new(0.0, 0.0, 20.0);

        app.update();

        assert_eq!(
            app.world().get::<AllySummonAi>(summon).unwrap().target,
            None
        );
    }

    #[test]
    fn ally_summon_drops_target_when_owner_moves_beyond_leash_range() {
        let mut app = setup_test_app();

        let summon = spawn_ally_summon(
            app.world_mut(),
            Vec3::ZERO,
            15.0,
        );

        let owner = app.world()
            .get::<SummonedCreature>(summon)
            .unwrap()
            .owner;

        let enemy = spawn_enemy(
            app.world_mut(),
            Vec3::new(0.0, 0.0, 5.0),
            100.0,
        );

        app.update();

        assert_eq!(
            app.world().get::<AllySummonAi>(summon).unwrap().target,
            Some(enemy)
        );

        app.world_mut()
            .get_mut::<SummonCombatProfile>(summon)
            .unwrap()
            .leash_range = 12.0;

        app.world_mut()
            .get_mut::<Transform>(owner)
            .unwrap()
            .translation = Vec3::new(0.0, 0.0, 13.0);

        app.update();

        assert_eq!(
            app.world().get::<AllySummonAi>(summon).unwrap().target,
            None
        );
    }

    #[test]
    fn ally_summon_reacquires_enemy_after_returning_within_leash_range() {
        let mut app = setup_test_app();

        let summon = spawn_ally_summon(
            app.world_mut(),
            Vec3::ZERO,
            15.0,
        );

        let owner = app.world()
            .get::<SummonedCreature>(summon)
            .unwrap()
            .owner;

        let enemy = spawn_enemy(
            app.world_mut(),
            Vec3::new(0.0, 0.0, 5.0),
            100.0,
        );

        app.world_mut()
            .get_mut::<SummonCombatProfile>(summon)
            .unwrap()
            .leash_range = 12.0;

        app.update();

        assert_eq!(
            app.world().get::<AllySummonAi>(summon).unwrap().target,
            Some(enemy)
        );

        app.world_mut()
            .get_mut::<Transform>(owner)
            .unwrap()
            .translation = Vec3::new(0.0, 0.0, 13.0);

        app.update();

        assert_eq!(
            app.world().get::<AllySummonAi>(summon).unwrap().target,
            None
        );

        app.world_mut()
            .get_mut::<Transform>(owner)
            .unwrap()
            .translation = Vec3::ZERO;

        app.update();

        assert_eq!(
            app.world().get::<AllySummonAi>(summon).unwrap().target,
            Some(enemy)
        );
    }

    #[test]
    fn ally_summon_drops_target_when_owner_is_missing() {
        let mut app = setup_test_app();

        let summon = spawn_ally_summon(
            app.world_mut(),
            Vec3::ZERO,
            15.0,
        );

        let owner = app.world()
            .get::<SummonedCreature>(summon)
            .unwrap()
            .owner;

        let enemy = spawn_enemy(
            app.world_mut(),
            Vec3::new(0.0, 0.0, 5.0),
            100.0,
        );

        app.update();

        assert_eq!(
            app.world().get::<AllySummonAi>(summon).unwrap().target,
            Some(enemy)
        );

        app.world_mut().despawn(owner);

        app.update();

        assert_eq!(
            app.world().get::<AllySummonAi>(summon).unwrap().target,
            None
        );
    }

    #[test]
    fn ally_summon_does_not_target_entities_without_enemy_marker() {
        let mut app = setup_test_app();

        let summon = spawn_ally_summon(
            app.world_mut(),
            Vec3::ZERO,
            15.0,
        );

        let player = app.world_mut().spawn((
            Transform::from_xyz(0.0, 0.0, 2.0),
            CombatStats {
                current_health: 100.0,
                max_health: 100.0,
                resource: 0.0,
            },
        )).id();

        app.update();

        assert_eq!(
            app.world().get::<AllySummonAi>(summon).unwrap().target,
            None
        );

        assert!(app.world().get::<Enemy>(player).is_none());
    }

    #[test]
    fn ally_summon_does_not_target_enemy_without_health_components() {
        let mut app = setup_test_app();

        let summon = spawn_ally_summon(
            app.world_mut(),
            Vec3::ZERO,
            15.0,
        );

        app.world_mut().spawn((
            Enemy,
            Transform::from_xyz(0.0, 0.0, 2.0),
        ));

        app.update();

        assert_eq!(
            app.world().get::<AllySummonAi>(summon).unwrap().target,
            None
        );
    }

    #[test]
    fn invalid_aggro_range_uses_default_range() {
        let mut app = setup_test_app();

        let summon = spawn_ally_summon(
            app.world_mut(),
            Vec3::ZERO,
            f32::NAN,
        );

        let enemy = spawn_enemy(
            app.world_mut(),
            Vec3::new(0.0, 0.0, 10.0),
            100.0,
        );

        app.update();

        assert_eq!(
            app.world().get::<AllySummonAi>(summon).unwrap().target,
            Some(enemy)
        );
    }
}
