use super::*;

#[test]
fn classic_pack_summon_imp_executes_end_to_end() {
    let mut database = ContentDatabase::new();

    crate::content::ContentPackLoader::load_ron(
        &mut database,
        include_str!("../../../../content/packs/classic_1_12_1.ron"),
    )
    .unwrap();

    let mut app = App::new();

    app.insert_resource(database.clone())
        .insert_resource(Time::<()>::default())
        .init_resource::<PendingCombatEffects>()
        .init_resource::<CombatEventQueue>()
        .init_resource::<PendingSummons>()
        .add_systems(
            Update,
            (effects::execute_pending_effects, spawn_pending_summons).chain(),
        );

    let owner_position = Vec3::new(4.0, 5.0, 6.0);

    let owner = app
        .world_mut()
        .spawn((
            Character,
            CombatLevel::new(60).unwrap(),
            CombatAttributes::default(),
            Transform::from_translation(owner_position),
        ))
        .id();

    let summon_effect_id = ContentId::new("classic_1_12_1:effect.summon_imp").unwrap();

    let resolved_count = app
        .world_mut()
        .resource_mut::<PendingCombatEffects>()
        .resolve_and_enqueue_from_database(
            &database,
            &summon_effect_id,
            Some(owner),
            EffectTargetSelection::Caster,
            &CombatAttributes::default(),
        )
        .unwrap();

    assert_eq!(resolved_count, 1);

    app.update();

    let world = app.world_mut();

    let mut summon_query = world.query_filtered::<Entity, With<AllySummon>>();

    let spawned_summons: Vec<Entity> = summon_query.iter(world).collect();

    assert_eq!(spawned_summons.len(), 1);

    let summon_entity = spawned_summons[0];

    let summon = world.get::<SummonedCreature>(summon_entity).unwrap();

    let level = world.get::<CombatLevel>(summon_entity).unwrap();

    let health = world.get::<HealthPool>(summon_entity).unwrap();

    let stats = world.get::<CombatStats>(summon_entity).unwrap();

    let profile = world.get::<SummonCombatProfile>(summon_entity).unwrap();

    let transform = world.get::<Transform>(summon_entity).unwrap();

    assert_eq!(summon.owner, owner);
    assert_eq!(summon.creature, creature_id());
    assert_eq!(summon.faction, SummonFaction::Allies);
    assert_eq!(summon.remaining_duration(), None);

    assert_eq!(level.value(), 60);

    assert_eq!(health.current(), 150.0);
    assert_eq!(health.maximum(), 150.0);

    assert_eq!(stats.current_health, 150.0);
    assert_eq!(stats.max_health, 150.0);

    assert_eq!(profile.attack_damage, 12.0);
    assert_eq!(profile.attack_interval_seconds, 2.0);
    assert_eq!(profile.melee_range, 2.5);
    assert_eq!(profile.aggro_range, 15.0);
    assert_eq!(profile.leash_range, 40.0);
    assert_eq!(profile.movement_speed, 5.0);

    assert_eq!(transform.translation, owner_position);

    assert!(world.get::<Enemy>(summon_entity).is_none());

    assert!(world.resource::<PendingSummons>().is_empty());
}

#[test]
fn classic_pack_imp_acquires_and_attacks_enemy_end_to_end() {
    let mut database = ContentDatabase::new();

    crate::content::ContentPackLoader::load_ron(
        &mut database,
        include_str!("../../../../content/packs/classic_1_12_1.ron"),
    )
    .unwrap();

    let mut app = App::new();

    app.insert_resource(database.clone())
        .insert_resource(Time::<()>::default())
        .init_resource::<PendingCombatEffects>()
        .init_resource::<CombatEventQueue>()
        .init_resource::<PendingSummons>()
        .add_systems(
            Update,
            (
                summon_lifecycle_system,
                effects::execute_pending_effects,
                spawn_pending_summons,
                enemy::enemy_aggro_system,
                enemy::enemy_attack_system,
                ally_summon_aggro_system,
                ally_summon_attack_system,
            )
                .chain(),
        );

    let owner = app
        .world_mut()
        .spawn((
            Character,
            CombatLevel::new(60).unwrap(),
            CombatAttributes::default(),
            Transform::IDENTITY,
        ))
        .id();

    let enemy = app
        .world_mut()
        .spawn((
            Enemy,
            EnemyAi::default(),
            CombatLevel::new(60).unwrap(),
            Transform::from_xyz(0.0, 0.0, 2.0),
            CombatStats {
                current_health: 100.0,
                max_health: 100.0,
                resource: 0.0,
            },
            HealthPool::new(100.0, 100.0).unwrap(),
        ))
        .id();

    let summon_effect_id = ContentId::new("classic_1_12_1:effect.summon_imp").unwrap();

    let resolved_count = app
        .world_mut()
        .resource_mut::<PendingCombatEffects>()
        .resolve_and_enqueue_from_database(
            &database,
            &summon_effect_id,
            Some(owner),
            EffectTargetSelection::Caster,
            &CombatAttributes::default(),
        )
        .unwrap();

    assert_eq!(resolved_count, 1);

    app.update();

    let summon_entity = {
        let world = app.world_mut();
        let mut summon_query = world.query_filtered::<Entity, With<AllySummon>>();

        summon_query.iter(world).next().unwrap()
    };

    let summon_ai = app.world().get::<AllySummonAi>(summon_entity).unwrap();

    assert_eq!(summon_ai.target, Some(enemy));

    advance_test_time(&mut app, 2.0);
    app.update();

    let enemy_health = app.world().get::<HealthPool>(enemy).unwrap();

    let enemy_stats = app.world().get::<CombatStats>(enemy).unwrap();

    assert_eq!(enemy_health.current(), 88.0);
    assert_eq!(enemy_stats.current_health, 88.0);

    let threat = app.world().get::<EnemyThreat>(enemy).unwrap();

    assert!(threat
        .values
        .get(&summon_entity)
        .is_some_and(|value| *value > 0.0));
}
