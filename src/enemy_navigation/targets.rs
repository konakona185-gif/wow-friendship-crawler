use bevy::prelude::*;
use bevy_landmass::prelude::AgentTarget3d;

use crate::combat::{
    AllySummon,
    AllySummonAi,
    Enemy,
    EnemyAi,
    EnemyHomePosition,
    SummonedCreature,
};
use crate::content::ContentId;

fn enemy_navigation_target(
    enemy_ai: &EnemyAi,
    home_position: Option<&EnemyHomePosition>,
) -> AgentTarget3d {
    if enemy_ai.returning_home {
        return home_position.map_or(
            AgentTarget3d::None,
            |home| AgentTarget3d::Point(home.0),
        );
    }

    enemy_ai.target.map_or(
        AgentTarget3d::None,
        AgentTarget3d::Entity,
    )
}

fn ally_summon_navigation_target(
    summon_ai: &AllySummonAi,
    summoned_creature: Option<&SummonedCreature>,
) -> AgentTarget3d {
    match summon_ai.target {
        Some(target) => AgentTarget3d::Entity(target),
        None => summoned_creature.map_or(
            AgentTarget3d::None,
            |summon| AgentTarget3d::Entity(summon.owner),
        ),
    }
}

fn navigation_target_matches(
    current: &AgentTarget3d,
    desired: &AgentTarget3d,
) -> bool {
    match (current, desired) {
        (AgentTarget3d::None, AgentTarget3d::None) => true,
        (
            AgentTarget3d::Entity(current),
            AgentTarget3d::Entity(desired),
        ) => current == desired,
        (
            AgentTarget3d::Point(current),
            AgentTarget3d::Point(desired),
        ) => current == desired,
        _ => false,
    }
}

pub(super) fn sync_enemy_navigation_targets(
    mut enemy_query: Query<
        (
            &EnemyAi,
            Option<&EnemyHomePosition>,
            &mut AgentTarget3d,
        ),
        (
            With<Enemy>,
            With<super::movement::EnemyNavigationAgent>,
            Without<AllySummon>,
        ),
    >,
    mut ally_summon_query: Query<
        (
            &AllySummonAi,
            Option<&SummonedCreature>,
            &mut AgentTarget3d,
        ),
        (
            With<AllySummon>,
            With<super::movement::AllySummonNavigationAgent>,
            Without<Enemy>,
        ),
    >,
) {
    for (enemy_ai, home_position, mut navigation_target) in &mut enemy_query {
        let desired_target =
            enemy_navigation_target(enemy_ai, home_position);

        if !navigation_target_matches(
            &navigation_target,
            &desired_target,
        ) {
            *navigation_target = desired_target;
        }
    }

    for (
        summon_ai,
        summoned_creature,
        mut navigation_target,
    ) in &mut ally_summon_query
    {
        let desired_target = ally_summon_navigation_target(
            summon_ai,
            summoned_creature,
        );

        if !navigation_target_matches(
            &navigation_target,
            &desired_target,
        ) {
            *navigation_target = desired_target;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_enemy_ai(
        target: Option<Entity>,
        returning_home: bool,
    ) -> EnemyAi {
        EnemyAi {
            target,
            aggro_range: 20.0,
            leash_range: 40.0,
            returning_home,
        }
    }

    fn test_summoned_creature(owner: Entity) -> SummonedCreature {
        SummonedCreature::new(
            owner,
            ContentId::new("test:creature.imp").unwrap(),
            crate::combat::SummonFaction::Allies,
            None,
        )
    }

    #[test]
    fn returning_enemy_targets_its_saved_home_position() {
        let mut world = World::new();
        let player = world.spawn_empty().id();

        let home = EnemyHomePosition(Vec3::new(12.0, 3.0, -8.0));
        let ai = test_enemy_ai(Some(player), true);

        let target = enemy_navigation_target(&ai, Some(&home));

        assert!(matches!(
            target,
            AgentTarget3d::Point(position) if position == home.0
        ));
    }

    #[test]
    fn returning_home_takes_priority_over_the_combat_target() {
        let mut world = World::new();
        let player = world.spawn_empty().id();

        let home = EnemyHomePosition(Vec3::new(5.0, 0.0, 9.0));
        let ai = test_enemy_ai(Some(player), true);

        let target = enemy_navigation_target(&ai, Some(&home));

        assert!(!matches!(
            target,
            AgentTarget3d::Entity(entity) if entity == player
        ));
    }

    #[test]
    fn active_combat_uses_the_player_entity_as_target() {
        let mut world = World::new();
        let player = world.spawn_empty().id();

        let ai = test_enemy_ai(Some(player), false);
        let target = enemy_navigation_target(&ai, None);

        assert!(matches!(
            target,
            AgentTarget3d::Entity(entity) if entity == player
        ));
    }

    #[test]
    fn enemy_without_combat_target_has_no_navigation_target() {
        let ai = test_enemy_ai(None, false);
        let target = enemy_navigation_target(&ai, None);

        assert!(matches!(target, AgentTarget3d::None));
    }

    #[test]
    fn returning_enemy_without_saved_home_has_no_navigation_target() {
        let ai = test_enemy_ai(None, true);
        let target = enemy_navigation_target(&ai, None);

        assert!(matches!(target, AgentTarget3d::None));
    }

    #[test]
    fn ally_summon_uses_its_enemy_as_navigation_target() {
        let mut world = World::new();
        let enemy = world.spawn_empty().id();

        let ai = AllySummonAi::new(15.0);
        let mut ai = ai;
        ai.target = Some(enemy);

        let target = ally_summon_navigation_target(
            &ai,
            None,
        );

        assert!(matches!(
            target,
            AgentTarget3d::Entity(entity) if entity == enemy
        ));
    }

    #[test]
    fn ally_summon_without_enemy_follows_its_owner() {
        let mut world = World::new();
        let owner = world.spawn_empty().id();

        let ai = AllySummonAi::new(15.0);
        let summoned_creature = test_summoned_creature(owner);

        let target = ally_summon_navigation_target(
            &ai,
            Some(&summoned_creature),
        );

        assert!(matches!(
            target,
            AgentTarget3d::Entity(entity) if entity == owner
        ));
    }

    #[test]
    fn ally_summon_without_owner_metadata_has_no_navigation_target() {
        let ai = AllySummonAi::new(15.0);

        let target = ally_summon_navigation_target(
            &ai,
            None,
        );

        assert!(matches!(target, AgentTarget3d::None));
    }

    #[test]
    fn ally_summon_target_updates_when_its_ai_target_changes() {
        let mut world = World::new();
        let first_enemy = world.spawn_empty().id();
        let second_enemy = world.spawn_empty().id();

        let mut ai = AllySummonAi::new(15.0);
        ai.target = Some(first_enemy);

        let first_target = ally_summon_navigation_target(
            &ai,
            None,
        );

        ai.target = Some(second_enemy);

        let second_target = ally_summon_navigation_target(
            &ai,
            None,
        );

        assert!(navigation_target_matches(
            &first_target,
            &AgentTarget3d::Entity(first_enemy),
        ));

        assert!(navigation_target_matches(
            &second_target,
            &AgentTarget3d::Entity(second_enemy),
        ));

        assert!(!navigation_target_matches(
            &first_target,
            &second_target,
        ));
    }

    #[test]
    fn navigation_target_comparison_detects_changes() {
        let mut world = World::new();
        let first = world.spawn_empty().id();
        let second = world.spawn_empty().id();

        assert!(navigation_target_matches(
            &AgentTarget3d::Entity(first),
            &AgentTarget3d::Entity(first),
        ));

        assert!(!navigation_target_matches(
            &AgentTarget3d::Entity(first),
            &AgentTarget3d::Entity(second),
        ));

        assert!(navigation_target_matches(
            &AgentTarget3d::Point(Vec3::ZERO),
            &AgentTarget3d::Point(Vec3::ZERO),
        ));

        assert!(!navigation_target_matches(
            &AgentTarget3d::Point(Vec3::ZERO),
            &AgentTarget3d::None,
        ));
    }
}
