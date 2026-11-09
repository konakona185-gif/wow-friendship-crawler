use super::*;

#[test]
fn summon_effect_creates_ally_summons_end_to_end() {
    let mut app = setup_summon_integration_app();

    let owner_position = Vec3::new(4.0, 5.0, 6.0);

    let owner = app
        .world_mut()
        .spawn((
            Character,
            CombatLevel::new(60).unwrap(),
            Transform::from_translation(owner_position),
        ))
        .id();

    enqueue_summon_effect(&mut app, owner);

    app.update();

    let world = app.world_mut();

    let mut summon_query = world.query_filtered::<Entity, With<AllySummon>>();

    let spawned_entities: Vec<Entity> = summon_query.iter(world).collect();

    assert_eq!(spawned_entities.len(), 2);

    for entity in spawned_entities {
        let summon = world.get::<SummonedCreature>(entity).unwrap();
        let level = world.get::<CombatLevel>(entity).unwrap();
        let stats = world.get::<CombatStats>(entity).unwrap();
        let health = world.get::<HealthPool>(entity).unwrap();
        let profile = world.get::<SummonCombatProfile>(entity).unwrap();
        let transform = world.get::<Transform>(entity).unwrap();

        assert_eq!(summon.owner, owner);
        assert_eq!(summon.faction, SummonFaction::Allies);
        assert_eq!(summon.creature, creature_id());
        assert_eq!(summon.remaining_duration(), Some(30.0));

        assert_eq!(level.value(), 60);

        assert_eq!(stats.current_health, 150.0);
        assert_eq!(stats.max_health, 150.0);
        assert_eq!(health.current(), 150.0);
        assert_eq!(health.maximum(), 150.0);

        assert_eq!(profile.attack_damage, 12.0);
        assert_eq!(profile.attack_interval_seconds, 2.0);
        assert_eq!(profile.melee_range, 2.5);
        assert_eq!(profile.aggro_range, 15.0);
        assert_eq!(profile.leash_range, 40.0);
        assert_eq!(profile.movement_speed, 5.0);

        assert_eq!(transform.translation, owner_position);

        assert!(world.get::<Enemy>(entity).is_none());
    }

    assert!(world.resource::<PendingSummons>().is_empty());
}

#[test]
fn summon_effect_creates_enemy_summons_end_to_end() {
    let mut app = setup_summon_integration_app();

    let owner_position = Vec3::new(8.0, 3.0, -4.0);

    let owner = app
        .world_mut()
        .spawn((
            Enemy,
            EnemyAi::default(),
            CombatLevel::new(42).unwrap(),
            Transform::from_translation(owner_position),
        ))
        .id();

    enqueue_summon_effect(&mut app, owner);

    app.update();

    let world = app.world_mut();

    let mut summon_query = world.query_filtered::<Entity, (With<Enemy>, With<SummonedCreature>)>();

    let spawned_entities: Vec<Entity> = summon_query.iter(world).collect();

    assert_eq!(spawned_entities.len(), 2);

    for entity in spawned_entities {
        let summon = world.get::<SummonedCreature>(entity).unwrap();
        let level = world.get::<CombatLevel>(entity).unwrap();
        let stats = world.get::<CombatStats>(entity).unwrap();
        let health = world.get::<HealthPool>(entity).unwrap();
        let profile = world.get::<SummonCombatProfile>(entity).unwrap();
        let ai = world.get::<EnemyAi>(entity).unwrap();
        let home = world.get::<super::EnemyHomePosition>(entity).unwrap();
        let transform = world.get::<Transform>(entity).unwrap();

        assert_eq!(summon.owner, owner);
        assert_eq!(summon.faction, SummonFaction::Enemies);
        assert_eq!(summon.creature, creature_id());
        assert_eq!(summon.remaining_duration(), Some(30.0));

        assert_eq!(level.value(), 42);

        assert_eq!(stats.current_health, 150.0);
        assert_eq!(stats.max_health, 150.0);
        assert_eq!(health.current(), 150.0);
        assert_eq!(health.maximum(), 150.0);

        assert_eq!(profile.attack_damage, 12.0);
        assert_eq!(profile.attack_interval_seconds, 2.0);
        assert_eq!(profile.melee_range, 2.5);
        assert_eq!(profile.aggro_range, 15.0);
        assert_eq!(profile.leash_range, 40.0);
        assert_eq!(profile.movement_speed, 5.0);

        assert_eq!(ai.aggro_range, 15.0);
        assert_eq!(ai.leash_range, 40.0);
        assert_eq!(home.0, owner_position);
        assert_eq!(transform.translation, owner_position);

        assert!(world.get::<AllySummon>(entity).is_none());
    }

    assert!(world.resource::<PendingSummons>().is_empty());
}

#[test]
fn displacement_effect_supports_both_summon_factions() {
    let mut app = setup_summon_integration_app();

    let ally_owner = app
        .world_mut()
        .spawn((
            Character,
            CombatLevel::new(60).unwrap(),
            Transform::from_xyz(5.0, 0.0, 0.0),
        ))
        .id();

    let enemy_owner = app
        .world_mut()
        .spawn((
            Enemy,
            EnemyAi::default(),
            CombatLevel::new(42).unwrap(),
            Transform::from_xyz(-5.0, 0.0, 0.0),
        ))
        .id();

    enqueue_summon_effect(&mut app, ally_owner);
    enqueue_summon_effect(&mut app, enemy_owner);

    app.update();

    let (ally_summons, enemy_summons) = {
        let world = app.world_mut();

        let mut ally_query = world.query_filtered::<Entity, With<AllySummon>>();

        let mut enemy_query =
            world.query_filtered::<Entity, (With<Enemy>, With<SummonedCreature>)>();

        (
            ally_query.iter(world).collect::<Vec<_>>(),
            enemy_query.iter(world).collect::<Vec<_>>(),
        )
    };

    assert_eq!(ally_summons.len(), 2);
    assert_eq!(enemy_summons.len(), 2);

    for entity in ally_summons.iter().chain(enemy_summons.iter()) {
        assert!(app.world().get::<PendingDisplacement>(*entity).is_some());
    }

    let displacement_source = app.world_mut().spawn(Transform::IDENTITY).id();

    for entity in &ally_summons {
        enqueue_displacement_effect(&mut app, displacement_source, *entity, 3.0);
    }

    for entity in &enemy_summons {
        enqueue_displacement_effect(&mut app, displacement_source, *entity, 3.0);
    }

    app.update();

    for entity in &ally_summons {
        let displacement = app.world().get::<PendingDisplacement>(*entity).unwrap();

        assert_eq!(displacement.pending_translation(), Vec3::X * 3.0);
    }

    for entity in &enemy_summons {
        let displacement = app.world().get::<PendingDisplacement>(*entity).unwrap();

        assert_eq!(displacement.pending_translation(), Vec3::NEG_X * 3.0);
    }
}
