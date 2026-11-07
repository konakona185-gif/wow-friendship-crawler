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
    effect_runtime,
    summons,
    AllySummon,
    CombatAttributes,
    CombatEventQueue,
    CombatLevel,
    CombatStats,
    EffectTargetSelection,
    Enemy,
    EnemyAi,
    HealthPool,
    PendingCombatEffects,
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
