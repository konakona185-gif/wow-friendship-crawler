use std::collections::HashSet;
use std::time::Duration;

use bevy::prelude::*;

use crate::content::ContentId;
use crate::player::Character;

use crate::combat::{
    AllySummonAi, CombatStats, EnemyAi, HealthPool, SummonedCreature, Target, TargetIndicator,
};

fn combatant_is_alive(stats: Option<&CombatStats>, health_pool: Option<&HealthPool>) -> bool {
    let current_health = health_pool
        .map(|pool| pool.current())
        .or_else(|| stats.map(|stats| stats.current_health));

    match current_health {
        Some(health) => health.is_finite() && health > 0.0,
        None => true,
    }
}

pub(in crate::combat) fn summon_lifecycle_system(
    time: Res<Time>,
    mut commands: Commands,
    mut summon_query: Query<(
        Entity,
        &mut SummonedCreature,
        Option<&CombatStats>,
        Option<&HealthPool>,
    )>,
    owner_query: Query<(Option<&CombatStats>, Option<&HealthPool>)>,
    mut enemy_ai_query: Query<&mut EnemyAi>,
    mut ally_summon_ai_query: Query<&mut AllySummonAi>,
    mut player_target_query: Query<&mut Target, With<Character>>,
    indicator_query: Query<(Entity, &TargetIndicator)>,
) {
    let delta = time.delta_secs();
    let mut summon_owners = Vec::new();
    let mut despawned_summons = HashSet::new();

    for (summon_entity, mut summon, summon_stats, summon_health_pool) in &mut summon_query {
        summon_owners.push((summon_entity, summon.owner));

        let owner_alive = owner_query
            .get(summon.owner)
            .is_ok_and(|(stats, health_pool)| combatant_is_alive(stats, health_pool));

        let summon_alive = combatant_is_alive(summon_stats, summon_health_pool);

        let expired = summon.tick_lifetime(delta);

        if !owner_alive || !summon_alive || expired {
            despawned_summons.insert(summon_entity);
        }
    }

    loop {
        let newly_despawned: Vec<Entity> = summon_owners
            .iter()
            .filter_map(|(summon_entity, owner)| {
                if despawned_summons.contains(owner) && !despawned_summons.contains(summon_entity) {
                    Some(*summon_entity)
                } else {
                    None
                }
            })
            .collect();

        if newly_despawned.is_empty() {
            break;
        }

        despawned_summons.extend(newly_despawned);
    }

    for summon_entity in despawned_summons {
        commands.entity(summon_entity).despawn();

        for mut enemy_ai in &mut enemy_ai_query {
            if enemy_ai.target == Some(summon_entity) {
                enemy_ai.target = None;
            }
        }

        for mut ally_summon_ai in &mut ally_summon_ai_query {
            if ally_summon_ai.target == Some(summon_entity) {
                ally_summon_ai.target = None;
            }
        }

        for mut player_target in &mut player_target_query {
            if player_target.entity == Some(summon_entity) {
                player_target.entity = None;
            }
        }

        for (indicator_entity, indicator) in &indicator_query {
            if indicator.enemy == summon_entity {
                commands.entity(indicator_entity).despawn();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn creature_id() -> ContentId {
        ContentId::new("test:creature.imp").unwrap()
    }

    fn setup_test_app() -> App {
        let mut app = App::new();

        app.insert_resource(Time::<()>::default())
            .add_systems(Update, summon_lifecycle_system);

        app
    }

    fn advance_test_time(app: &mut App, seconds: f32) {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(seconds));
    }

    fn spawn_living_owner(world: &mut World) -> Entity {
        world
            .spawn((
                Character,
                CombatStats {
                    current_health: 100.0,
                    max_health: 100.0,
                    resource: 0.0,
                },
                HealthPool::new(100.0, 100.0).unwrap(),
            ))
            .id()
    }

    fn spawn_summon(
        world: &mut World,
        owner: Entity,
        duration: Option<f32>,
        health: f32,
    ) -> Entity {
        world
            .spawn((
                SummonedCreature::new(
                    owner,
                    creature_id(),
                    super::super::SummonFaction::Allies,
                    duration,
                ),
                CombatStats {
                    current_health: health,
                    max_health: 100.0,
                    resource: 0.0,
                },
                HealthPool::new(health, 100.0).unwrap(),
                Transform::IDENTITY,
            ))
            .id()
    }

    #[test]
    fn timed_summon_expires_after_its_duration() {
        let mut app = setup_test_app();

        let owner = spawn_living_owner(app.world_mut());

        let summon = spawn_summon(app.world_mut(), owner, Some(5.0), 100.0);

        advance_test_time(&mut app, 4.0);
        app.update();

        assert!(app.world().get::<SummonedCreature>(summon).is_some());

        assert_eq!(
            app.world()
                .get::<SummonedCreature>(summon)
                .unwrap()
                .remaining_duration(),
            Some(1.0)
        );

        advance_test_time(&mut app, 1.0);
        app.update();

        assert!(app.world().get::<SummonedCreature>(summon).is_none());
    }

    #[test]
    fn permanent_summon_does_not_expire() {
        let mut app = setup_test_app();

        let owner = spawn_living_owner(app.world_mut());

        let summon = spawn_summon(app.world_mut(), owner, None, 100.0);

        advance_test_time(&mut app, 120.0);
        app.update();

        assert!(app.world().get::<SummonedCreature>(summon).is_some());
    }

    #[test]
    fn defeated_summon_is_despawned() {
        let mut app = setup_test_app();

        let owner = spawn_living_owner(app.world_mut());

        let summon = spawn_summon(app.world_mut(), owner, None, 0.0);

        app.update();

        assert!(app.world().get::<SummonedCreature>(summon).is_none());
    }

    #[test]
    fn defeated_summon_is_despawned_when_stats_are_zero() {
        let mut app = setup_test_app();

        let owner = spawn_living_owner(app.world_mut());

        let summon = world_spawn_summon_with_only_stats(app.world_mut(), owner, 0.0);

        app.update();

        assert!(app.world().get::<SummonedCreature>(summon).is_none());
    }

    fn world_spawn_summon_with_only_stats(world: &mut World, owner: Entity, health: f32) -> Entity {
        world
            .spawn((
                SummonedCreature::new(
                    owner,
                    creature_id(),
                    super::super::SummonFaction::Allies,
                    None,
                ),
                CombatStats {
                    current_health: health,
                    max_health: 100.0,
                    resource: 0.0,
                },
            ))
            .id()
    }

    #[test]
    fn summon_is_despawned_when_owner_is_removed() {
        let mut app = setup_test_app();

        let owner = spawn_living_owner(app.world_mut());

        let summon = spawn_summon(app.world_mut(), owner, None, 100.0);

        app.world_mut().despawn(owner);

        app.update();

        assert!(app.world().get::<SummonedCreature>(summon).is_none());
    }

    #[test]
    fn summon_is_despawned_when_owner_dies() {
        let mut app = setup_test_app();

        let owner = spawn_living_owner(app.world_mut());

        let summon = spawn_summon(app.world_mut(), owner, None, 100.0);

        app.world_mut()
            .get_mut::<HealthPool>(owner)
            .unwrap()
            .set_current(0.0)
            .unwrap();

        app.update();

        assert!(app.world().get::<SummonedCreature>(summon).is_none());
    }

    #[test]
    fn health_pool_overrides_stale_owner_stats() {
        let mut app = setup_test_app();

        let owner = spawn_living_owner(app.world_mut());

        let summon = spawn_summon(app.world_mut(), owner, None, 100.0);

        app.world_mut()
            .get_mut::<HealthPool>(owner)
            .unwrap()
            .set_current(0.0)
            .unwrap();

        app.update();

        assert!(app.world().get::<SummonedCreature>(summon).is_none());
    }

    #[test]
    fn summon_without_health_components_is_not_treated_as_defeated() {
        let mut app = setup_test_app();

        let owner = spawn_living_owner(app.world_mut());

        let summon = app
            .world_mut()
            .spawn((SummonedCreature::new(
                owner,
                creature_id(),
                super::super::SummonFaction::Allies,
                None,
            ),))
            .id();

        app.update();

        assert!(app.world().get::<SummonedCreature>(summon).is_some());
    }

    #[test]
    fn removing_summon_clears_combat_target_references() {
        let mut app = setup_test_app();

        let owner = spawn_living_owner(app.world_mut());

        let summon = spawn_summon(app.world_mut(), owner, None, 0.0);

        let enemy = app
            .world_mut()
            .spawn((
                crate::combat::Enemy,
                EnemyAi {
                    target: Some(summon),
                    ..EnemyAi::default()
                },
            ))
            .id();

        let ally_summon = app
            .world_mut()
            .spawn((
                crate::combat::AllySummon,
                AllySummonAi {
                    target: Some(summon),
                    aggro_range: 15.0,
                },
            ))
            .id();

        let player = app
            .world_mut()
            .spawn((
                Character,
                Target {
                    entity: Some(summon),
                },
            ))
            .id();

        let indicator = app
            .world_mut()
            .spawn(TargetIndicator { enemy: summon })
            .id();

        app.update();

        assert!(app.world().get::<SummonedCreature>(summon).is_none());

        assert_eq!(app.world().get::<EnemyAi>(enemy).unwrap().target, None);

        assert_eq!(
            app.world().get::<AllySummonAi>(ally_summon).unwrap().target,
            None
        );

        assert_eq!(app.world().get::<Target>(player).unwrap().entity, None);

        assert!(app.world().get::<TargetIndicator>(indicator).is_none());
    }

    #[test]
    fn all_descendants_are_despawned_when_an_ancestor_expires() {
        let mut app = setup_test_app();

        let owner = spawn_living_owner(app.world_mut());

        let parent = spawn_summon(app.world_mut(), owner, Some(1.0), 100.0);

        let child = spawn_summon(app.world_mut(), parent, None, 100.0);

        let grandchild = spawn_summon(app.world_mut(), child, None, 100.0);

        advance_test_time(&mut app, 1.0);
        app.update();

        assert!(app.world().get::<SummonedCreature>(parent).is_none());

        assert!(app.world().get::<SummonedCreature>(child).is_none());

        assert!(app.world().get::<SummonedCreature>(grandchild).is_none());
    }
}
