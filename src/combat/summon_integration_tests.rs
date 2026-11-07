use std::time::Duration;

use bevy::prelude::*;

use crate::content::{
    ContentDatabase,
    ContentId,
    ContentMetadata,
    ContentPackManifest,
    CreatureDefinition,
    EffectDefinition,
    EffectKind,
    EffectTarget,
    EffectTrigger,
};
use crate::player::Character;

use super::{
    enemy_ai,
    enemy_attacks,
    effect_runtime,
    summon_ai,
    summon_attacks,
    summon_lifecycle,
    summons,
    AllySummon,
    CombatAttributes,
    CombatEventQueue,
    CombatLevel,
    CombatStats,
    EffectTargetSelection,
    Enemy,
    EnemyAi,
    EnemyThreat,
    HealthPool,
    PendingCombatEffects,
    PendingDisplacement,
    ResolvedEffect,
    ResolvedEffectOperation,
    SummonCombatProfile,
    SummonFaction,
    SummonedCreature,
};

fn creature_id() -> ContentId {
    ContentId::new("classic_1_12_1:creature.imp").unwrap()
}

fn effect_id() -> ContentId {
    ContentId::new("classic_1_12_1:effect.summon_imp").unwrap()
}

fn pack_id() -> ContentId {
    ContentId::new("pack:classic_1_12_1").unwrap()
}

fn summon_test_database() -> ContentDatabase {
    let mut database = ContentDatabase::new();

    let pack_id = pack_id();
    let creature_id = creature_id();

    database
        .register_pack(ContentPackManifest {
            id: pack_id.clone(),
            name: "Classic 1.12.1".to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            dependencies: Vec::new(),
        })
        .unwrap();

    database
        .register_creature(CreatureDefinition {
            metadata: ContentMetadata {
                id: creature_id.clone(),
                pack_id: pack_id.clone(),
                name: "Imp".to_string(),
                description: String::new(),
                icon: None,
                tags: vec!["summon".to_string()],
            },
            health: 150.0,
            attack_damage: 12.0,
            attack_interval_seconds: 2.0,
            melee_range: 2.5,
            aggro_range: 15.0,
            leash_range: 40.0,
            movement_speed: 5.0,
        })
        .unwrap();

    database
        .register_effect(EffectDefinition {
            metadata: ContentMetadata {
                id: effect_id(),
                pack_id,
                name: "Summon Imp".to_string(),
                description: String::new(),
                icon: None,
                tags: vec!["summon".to_string()],
            },
            trigger: EffectTrigger::OnCast,
            target: EffectTarget::Caster,
            kind: EffectKind::Summon {
                creature: creature_id,
                count: 2,
                duration_seconds: Some(30.0),
            },
        })
        .unwrap();

    database.validate().unwrap();

    database
}

fn setup_summon_integration_app() -> App {
    let mut app = App::new();

    app.insert_resource(summon_test_database())
        .insert_resource(Time::<()>::default())
        .init_resource::<PendingCombatEffects>()
        .init_resource::<CombatEventQueue>()
        .init_resource::<summons::PendingSummons>()
        .add_systems(
            Update,
            (
                summon_lifecycle::summon_lifecycle_system,
                effect_runtime::execute_pending_effects,
                summons::spawn_pending_summons,
                enemy_ai::enemy_aggro_system,
                enemy_attacks::enemy_attack_system,
                summon_ai::ally_summon_aggro_system,
                summon_attacks::ally_summon_attack_system,
            )
                .chain(),
        );

    app
}

fn enqueue_summon_effect(
    app: &mut App,
    owner: Entity,
) {
    let database = app
        .world()
        .resource::<ContentDatabase>()
        .clone();

    app.world_mut()
        .resource_mut::<PendingCombatEffects>()
        .resolve_and_enqueue_from_database(
            &database,
            &effect_id(),
            Some(owner),
            EffectTargetSelection::Caster,
            &CombatAttributes::default(),
        )
        .unwrap();
}

fn enqueue_displacement_effect(
    app: &mut App,
    source: Entity,
    target: Entity,
    distance: f32,
) {
    app.world_mut()
        .resource_mut::<PendingCombatEffects>()
        .enqueue(ResolvedEffect {
            effect: ContentId::new(
                "classic_1_12_1:effect.displacement_test",
            )
            .unwrap(),
            trigger: EffectTrigger::OnCast,
            source: Some(source),
            target,
            operation: ResolvedEffectOperation::Displacement {
                distance,
            },
        });
}

fn advance_test_time(
    app: &mut App,
    seconds: f32,
) {
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(seconds));
}

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

    let mut summon_query =
        world.query_filtered::<Entity, With<AllySummon>>();

    let spawned_entities: Vec<Entity> =
        summon_query.iter(world).collect();

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

    assert!(
        world
            .resource::<summons::PendingSummons>()
            .is_empty()
    );
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

    let mut summon_query = world.query_filtered::<
        Entity,
        (With<Enemy>, With<SummonedCreature>),
    >();

    let spawned_entities: Vec<Entity> =
        summon_query.iter(world).collect();

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

    assert!(
        world
            .resource::<summons::PendingSummons>()
            .is_empty()
    );
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

        let mut ally_query =
            world.query_filtered::<Entity, With<AllySummon>>();

        let mut enemy_query = world.query_filtered::<
            Entity,
            (With<Enemy>, With<SummonedCreature>),
        >();

        (
            ally_query.iter(world).collect::<Vec<_>>(),
            enemy_query.iter(world).collect::<Vec<_>>(),
        )
    };

    assert_eq!(ally_summons.len(), 2);
    assert_eq!(enemy_summons.len(), 2);

    for entity in ally_summons.iter().chain(enemy_summons.iter()) {
        assert!(
            app.world()
                .get::<PendingDisplacement>(*entity)
                .is_some()
        );
    }

    let displacement_source = app
        .world_mut()
        .spawn(Transform::IDENTITY)
        .id();

    for entity in &ally_summons {
        enqueue_displacement_effect(
            &mut app,
            displacement_source,
            *entity,
            3.0,
        );
    }

    for entity in &enemy_summons {
        enqueue_displacement_effect(
            &mut app,
            displacement_source,
            *entity,
            3.0,
        );
    }

    app.update();

    for entity in &ally_summons {
        let displacement = app
            .world()
            .get::<PendingDisplacement>(*entity)
            .unwrap();

        assert_eq!(
            displacement.pending_translation(),
            Vec3::X * 3.0
        );
    }

    for entity in &enemy_summons {
        let displacement = app
            .world()
            .get::<PendingDisplacement>(*entity)
            .unwrap();

        assert_eq!(
            displacement.pending_translation(),
            Vec3::NEG_X * 3.0
        );
    }
}

#[test]
fn allied_summon_attacks_generate_threat_against_the_enemy() {
    let mut app = setup_summon_integration_app();

    let owner = app
        .world_mut()
        .spawn((
            Character,
            CombatLevel::new(60).unwrap(),
            Transform::IDENTITY,
            CombatStats {
                current_health: 100.0,
                max_health: 100.0,
                resource: 0.0,
            },
            HealthPool::new(100.0, 100.0).unwrap(),
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

    enqueue_summon_effect(&mut app, owner);

    app.update();

    let summon_entities = {
        let world = app.world_mut();
        let mut summon_query =
            world.query_filtered::<Entity, With<AllySummon>>();

        summon_query.iter(world).collect::<Vec<_>>()
    };

    assert_eq!(summon_entities.len(), 2);

    app.update();

    advance_test_time(&mut app, 2.0);
    app.update();

    let threat = app.world().get::<EnemyThreat>(enemy).unwrap();

    assert_eq!(threat.values.len(), summon_entities.len());

    for summon in &summon_entities {
        assert!(
            threat
                .values
                .get(summon)
                .is_some_and(|value| *value > 0.0)
        );
    }

    assert!(
        app.world()
            .get::<HealthPool>(enemy)
            .unwrap()
            .current()
            < 100.0
    );

    app.update();

    let enemy_target = app
        .world()
        .get::<EnemyAi>(enemy)
        .unwrap()
        .target;

    assert!(
        enemy_target.is_some_and(|target| {
            summon_entities.contains(&target)
        })
    );
}

#[test]
fn enemy_summons_acquire_and_attack_a_player_end_to_end() {
    let mut app = setup_summon_integration_app();

    let owner = app
        .world_mut()
        .spawn((
            Enemy,
            EnemyAi {
                returning_home: true,
                ..Default::default()
            },
            CombatLevel::new(60).unwrap(),
            Transform::IDENTITY,
        ))
        .id();

    let player = app
        .world_mut()
        .spawn((
            Character,
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

    enqueue_summon_effect(&mut app, owner);

    app.update();

    let summon_entities = {
        let world = app.world_mut();
        let mut summon_query = world.query_filtered::<
            Entity,
            (With<Enemy>, With<SummonedCreature>),
        >();

        summon_query.iter(world).collect::<Vec<_>>()
    };

    assert_eq!(summon_entities.len(), 2);

    for summon in &summon_entities {
        let ai = app.world().get::<EnemyAi>(*summon).unwrap();

        assert_eq!(ai.target, Some(player));
        assert!(
            app.world()
                .get::<AllySummon>(*summon)
                .is_none()
        );
    }

    advance_test_time(&mut app, 2.0);
    app.update();

    let player_health = app.world().get::<HealthPool>(player).unwrap();
    let player_stats = app.world().get::<CombatStats>(player).unwrap();

    assert_eq!(player_health.current(), 76.0);
    assert_eq!(player_stats.current_health, 76.0);
}

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
        let mut summon_query = world.query_filtered::<
            Entity,
            With<SummonedCreature>,
        >();

        summon_query.iter(world).collect::<Vec<_>>()
    };

    assert_eq!(spawned_summons.len(), 2);

    advance_test_time(&mut app, 30.0);
    app.update();

    let remaining_summons = {
        let world = app.world_mut();
        let mut summon_query = world.query_filtered::<
            Entity,
            With<SummonedCreature>,
        >();

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
        let mut summon_query = world.query_filtered::<
            Entity,
            With<SummonedCreature>,
        >();

        summon_query.iter(world).collect::<Vec<_>>()
    };

    assert_eq!(spawned_summons.len(), 2);

    app.world_mut().despawn(owner);
    app.update();

    let remaining_summons = {
        let world = app.world_mut();
        let mut summon_query = world.query_filtered::<
            Entity,
            With<SummonedCreature>,
        >();

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
        let mut summon_query =
            world.query_filtered::<Entity, With<AllySummon>>();

        summon_query.iter(world).next().unwrap()
    };

    enqueue_summon_effect(&mut app, parent_summon);

    app.update();

    let child_summons = {
        let world = app.world_mut();
        let mut summon_query =
            world.query::<(Entity, &SummonedCreature)>();

        summon_query
            .iter(world)
            .filter_map(|(entity, summon)| {
                (summon.owner == parent_summon).then_some(entity)
            })
            .collect::<Vec<_>>()
    };

    assert_eq!(child_summons.len(), 2);

    for child in &child_summons {
        let summon = app
            .world()
            .get::<SummonedCreature>(*child)
            .unwrap();

        let level = app
            .world()
            .get::<CombatLevel>(*child)
            .unwrap();

        assert_eq!(summon.owner, parent_summon);
        assert_eq!(summon.faction, SummonFaction::Allies);
        assert_eq!(level.value(), 60);

        assert!(
            app.world()
                .get::<AllySummon>(*child)
                .is_some()
        );

        assert!(
            app.world()
                .get::<Enemy>(*child)
                .is_none()
        );
    }

    let total_summons = {
        let world = app.world_mut();
        let mut summon_query =
            world.query_filtered::<Entity, With<SummonedCreature>>();

        summon_query.iter(world).count()
    };

    assert_eq!(total_summons, 4);

    app.world_mut().despawn(root_owner);
    app.update();

    let remaining_summons = {
        let world = app.world_mut();
        let mut summon_query =
            world.query_filtered::<Entity, With<SummonedCreature>>();

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
        let mut summon_query = world.query_filtered::<
            Entity,
            (With<Enemy>, With<SummonedCreature>),
        >();

        summon_query.iter(world).next().unwrap()
    };

    enqueue_summon_effect(&mut app, parent_summon);

    app.update();

    let child_summons = {
        let world = app.world_mut();
        let mut summon_query =
            world.query::<(Entity, &SummonedCreature)>();

        summon_query
            .iter(world)
            .filter_map(|(entity, summon)| {
                (summon.owner == parent_summon).then_some(entity)
            })
            .collect::<Vec<_>>()
    };

    assert_eq!(child_summons.len(), 2);

    for child in &child_summons {
        let summon = app
            .world()
            .get::<SummonedCreature>(*child)
            .unwrap();

        let level = app
            .world()
            .get::<CombatLevel>(*child)
            .unwrap();

        assert_eq!(summon.owner, parent_summon);
        assert_eq!(summon.faction, SummonFaction::Enemies);
        assert_eq!(level.value(), 60);

        assert!(
            app.world()
                .get::<Enemy>(*child)
                .is_some()
        );

        assert!(
            app.world()
                .get::<AllySummon>(*child)
                .is_none()
        );
    }

    let total_summons = {
        let world = app.world_mut();
        let mut summon_query =
            world.query_filtered::<Entity, With<SummonedCreature>>();

        summon_query.iter(world).count()
    };

    assert_eq!(total_summons, 4);

    app.world_mut().despawn(root_owner);
    app.update();

    let remaining_summons = {
        let world = app.world_mut();
        let mut summon_query =
            world.query_filtered::<Entity, With<SummonedCreature>>();

        summon_query.iter(world).count()
    };

    assert_eq!(remaining_summons, 0);
}

#[test]
fn classic_pack_summon_imp_executes_end_to_end() {
    let mut database = ContentDatabase::new();

    crate::content::ContentPackLoader::load_ron(
        &mut database,
        include_str!("../../content/packs/classic_1_12_1.ron"),
    )
    .unwrap();

    let mut app = App::new();

    app.insert_resource(database.clone())
        .insert_resource(Time::<()>::default())
        .init_resource::<PendingCombatEffects>()
        .init_resource::<CombatEventQueue>()
        .init_resource::<summons::PendingSummons>()
        .add_systems(
            Update,
            (
                effect_runtime::execute_pending_effects,
                summons::spawn_pending_summons,
            )
                .chain(),
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

    let summon_effect_id = ContentId::new(
        "classic_1_12_1:effect.summon_imp",
    )
    .unwrap();

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

    let mut summon_query =
        world.query_filtered::<Entity, With<AllySummon>>();

    let spawned_summons: Vec<Entity> =
        summon_query.iter(world).collect();

    assert_eq!(spawned_summons.len(), 1);

    let summon_entity = spawned_summons[0];

    let summon = world
        .get::<SummonedCreature>(summon_entity)
        .unwrap();

    let level = world
        .get::<CombatLevel>(summon_entity)
        .unwrap();

    let health = world
        .get::<HealthPool>(summon_entity)
        .unwrap();

    let stats = world
        .get::<CombatStats>(summon_entity)
        .unwrap();

    let profile = world
        .get::<SummonCombatProfile>(summon_entity)
        .unwrap();

    let transform = world
        .get::<Transform>(summon_entity)
        .unwrap();

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

    assert!(
        world.get::<Enemy>(summon_entity).is_none()
    );

    assert!(
        world
            .resource::<summons::PendingSummons>()
            .is_empty()
    );
}

#[test]
fn classic_pack_imp_acquires_and_attacks_enemy_end_to_end() {
    let mut database = ContentDatabase::new();

    crate::content::ContentPackLoader::load_ron(
        &mut database,
        include_str!("../../content/packs/classic_1_12_1.ron"),
    )
    .unwrap();

    let mut app = App::new();

    app.insert_resource(database.clone())
        .insert_resource(Time::<()>::default())
        .init_resource::<PendingCombatEffects>()
        .init_resource::<CombatEventQueue>()
        .init_resource::<summons::PendingSummons>()
        .add_systems(
            Update,
            (
                summon_lifecycle::summon_lifecycle_system,
                effect_runtime::execute_pending_effects,
                summons::spawn_pending_summons,
                enemy_ai::enemy_aggro_system,
                enemy_attacks::enemy_attack_system,
                summon_ai::ally_summon_aggro_system,
                summon_attacks::ally_summon_attack_system,
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

    let summon_effect_id = ContentId::new(
        "classic_1_12_1:effect.summon_imp",
    )
    .unwrap();

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
        let mut summon_query =
            world.query_filtered::<Entity, With<AllySummon>>();

        summon_query.iter(world).next().unwrap()
    };

    let summon_ai = app
        .world()
        .get::<summons::AllySummonAi>(summon_entity)
        .unwrap();

    assert_eq!(summon_ai.target, Some(enemy));

    advance_test_time(&mut app, 2.0);
    app.update();

    let enemy_health = app
        .world()
        .get::<HealthPool>(enemy)
        .unwrap();

    let enemy_stats = app
        .world()
        .get::<CombatStats>(enemy)
        .unwrap();

    assert_eq!(enemy_health.current(), 88.0);
    assert_eq!(enemy_stats.current_health, 88.0);

    let threat = app
        .world()
        .get::<EnemyThreat>(enemy)
        .unwrap();

    assert!(
        threat
            .values
            .get(&summon_entity)
            .is_some_and(|value| *value > 0.0)
    );
}
