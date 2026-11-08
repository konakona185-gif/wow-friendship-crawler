use super::*;

#[test]
fn timed_summons_expire_after_their_duration_end_to_end() {
    let mut app = setup_summon_integration_app();

    let owner = app
        .world_mut()
        .spawn((
            Character,
            CombatLevel::new(60).unwrap(),
            Transform::IDENTITY,
        ))
        .id();

    enqueue_summon_effect(&mut app, owner);

    app.update();

    let spawned_summons = {
        let world = app.world_mut();
        let mut summon_query = world.query_filtered::<Entity, With<SummonedCreature>>();

        summon_query.iter(world).collect::<Vec<_>>()
    };

    assert_eq!(spawned_summons.len(), 2);

    advance_test_time(&mut app, 30.0);
    app.update();

    let remaining_summons = {
        let world = app.world_mut();
        let mut summon_query = world.query_filtered::<Entity, With<SummonedCreature>>();

        summon_query.iter(world).count()
    };

    assert_eq!(remaining_summons, 0);
}

#[test]
fn summons_are_despawned_when_their_owner_is_removed_end_to_end() {
    let mut app = setup_summon_integration_app();

    let owner = app
        .world_mut()
        .spawn((
            Character,
            CombatLevel::new(60).unwrap(),
            Transform::IDENTITY,
        ))
        .id();

    enqueue_summon_effect(&mut app, owner);

    app.update();

    let spawned_summons = {
        let world = app.world_mut();
        let mut summon_query = world.query_filtered::<Entity, With<SummonedCreature>>();

        summon_query.iter(world).collect::<Vec<_>>()
    };

    assert_eq!(spawned_summons.len(), 2);

    app.world_mut().despawn(owner);
    app.update();

    let remaining_summons = {
        let world = app.world_mut();
        let mut summon_query = world.query_filtered::<Entity, With<SummonedCreature>>();

        summon_query.iter(world).count()
    };

    assert_eq!(remaining_summons, 0);
}

#[test]
fn recursive_summons_inherit_faction_and_are_cleaned_up_with_root_owner_end_to_end() {
    let mut app = setup_summon_integration_app();

    let root_owner = app
        .world_mut()
        .spawn((
            Character,
            CombatLevel::new(60).unwrap(),
            Transform::IDENTITY,
        ))
        .id();

    enqueue_summon_effect(&mut app, root_owner);

    app.update();

    let parent_summon = {
        let world = app.world_mut();
        let mut summon_query = world.query_filtered::<Entity, With<AllySummon>>();

        summon_query.iter(world).next().unwrap()
    };

    enqueue_summon_effect(&mut app, parent_summon);

    app.update();

    let child_summons = {
        let world = app.world_mut();
        let mut summon_query = world.query::<(Entity, &SummonedCreature)>();

        summon_query
            .iter(world)
            .filter_map(|(entity, summon)| (summon.owner == parent_summon).then_some(entity))
            .collect::<Vec<_>>()
    };

    assert_eq!(child_summons.len(), 2);

    for child in &child_summons {
        let summon = app.world().get::<SummonedCreature>(*child).unwrap();

        let level = app.world().get::<CombatLevel>(*child).unwrap();

        assert_eq!(summon.owner, parent_summon);
        assert_eq!(summon.faction, SummonFaction::Allies);
        assert_eq!(level.value(), 60);

        assert!(app.world().get::<AllySummon>(*child).is_some());

        assert!(app.world().get::<Enemy>(*child).is_none());
    }

    let total_summons = {
        let world = app.world_mut();
        let mut summon_query = world.query_filtered::<Entity, With<SummonedCreature>>();

        summon_query.iter(world).count()
    };

    assert_eq!(total_summons, 4);

    app.world_mut().despawn(root_owner);
    app.update();

    let remaining_summons = {
        let world = app.world_mut();
        let mut summon_query = world.query_filtered::<Entity, With<SummonedCreature>>();

        summon_query.iter(world).count()
    };

    assert_eq!(remaining_summons, 0);
}

#[test]
fn recursive_enemy_summons_inherit_faction_and_are_cleaned_up_with_root_owner_end_to_end() {
    let mut app = setup_summon_integration_app();

    let root_owner = app
        .world_mut()
        .spawn((
            Enemy,
            EnemyAi::default(),
            CombatLevel::new(60).unwrap(),
            Transform::IDENTITY,
        ))
        .id();

    enqueue_summon_effect(&mut app, root_owner);

    app.update();

    let parent_summon = {
        let world = app.world_mut();
        let mut summon_query =
            world.query_filtered::<Entity, (With<Enemy>, With<SummonedCreature>)>();

        summon_query.iter(world).next().unwrap()
    };

    enqueue_summon_effect(&mut app, parent_summon);

    app.update();

    let child_summons = {
        let world = app.world_mut();
        let mut summon_query = world.query::<(Entity, &SummonedCreature)>();

        summon_query
            .iter(world)
            .filter_map(|(entity, summon)| (summon.owner == parent_summon).then_some(entity))
            .collect::<Vec<_>>()
    };

    assert_eq!(child_summons.len(), 2);

    for child in &child_summons {
        let summon = app.world().get::<SummonedCreature>(*child).unwrap();

        let level = app.world().get::<CombatLevel>(*child).unwrap();

        assert_eq!(summon.owner, parent_summon);
        assert_eq!(summon.faction, SummonFaction::Enemies);
        assert_eq!(level.value(), 60);

        assert!(app.world().get::<Enemy>(*child).is_some());

        assert!(app.world().get::<AllySummon>(*child).is_none());
    }

    let total_summons = {
        let world = app.world_mut();
        let mut summon_query = world.query_filtered::<Entity, With<SummonedCreature>>();

        summon_query.iter(world).count()
    };

    assert_eq!(total_summons, 4);

    app.world_mut().despawn(root_owner);
    app.update();

    let remaining_summons = {
        let world = app.world_mut();
        let mut summon_query = world.query_filtered::<Entity, With<SummonedCreature>>();

        summon_query.iter(world).count()
    };

    assert_eq!(remaining_summons, 0);
}
