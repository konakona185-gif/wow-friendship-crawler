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

use super::*;
use crate::combat::{
    effects,
    enemy,
    CombatEventQueue,
    EffectTargetSelection,
    EnemyThreat,
    PendingCombatEffects,
    PendingDisplacement,
    ResolvedEffect,
    ResolvedEffectOperation,
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

mod spawn_tests;
mod combat_tests;
mod lifecycle_tests;
mod classic_pack_tests;
