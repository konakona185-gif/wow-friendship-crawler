use super::*;

use crate::combat::{
    CombatAttributes, CombatEventQueue, EffectTargetSelection, ResolvedEffect,
    ResolvedEffectOperation,
};

use crate::content::{
    ContentDatabase, ContentMetadata, ContentPackManifest, CreatureDefinition, EffectDefinition,
    EffectKind, EffectTarget, EffectTrigger,
};

use super::super::PendingCombatEffects;

fn creature_id() -> ContentId {
    ContentId::new("classic_1_12_1:creature.imp").unwrap()
}

fn effect_id() -> ContentId {
    ContentId::new("classic_1_12_1:effect.summon_imp").unwrap()
}

fn pack_id() -> ContentId {
    ContentId::new("pack:classic_1_12_1").unwrap()
}

fn spawn_test_app() -> App {
    let database = summon_test_database(EffectTarget::Caster, true);

    let mut app = App::new();

    app.insert_resource(database)
        .init_resource::<PendingSummons>()
        .add_systems(Update, spawn_pending_summons);

    app
}

fn enqueue_spawn_request(app: &mut App, owner: Entity, count: u32) {
    app.world_mut()
        .resource_mut::<PendingSummons>()
        .enqueue(PendingSummon {
            owner,
            creature: creature_id(),
            spawn_position: Vec3::new(4.0, 2.0, -3.0),
            count,
            duration_seconds: Some(30.0),
        });
}

fn summon_test_database(target: EffectTarget, register_creature: bool) -> ContentDatabase {
    let mut database = ContentDatabase::new();

    let pack_id = pack_id();
    let creature_id = creature_id();
    let effect_id = effect_id();

    database
        .register_pack(ContentPackManifest {
            id: pack_id.clone(),
            name: "Classic 1.12.1".to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            dependencies: Vec::new(),
        })
        .unwrap();

    if register_creature {
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
    }

    database
        .register_effect(EffectDefinition {
            metadata: ContentMetadata {
                id: effect_id,
                pack_id,
                name: "Summon Imp".to_string(),
                description: String::new(),
                icon: None,
                tags: vec!["summon".to_string()],
            },
            trigger: EffectTrigger::OnCast,
            target,
            kind: EffectKind::Summon {
                creature: creature_id,
                count: 2,
                duration_seconds: Some(30.0),
            },
        })
        .unwrap();

    if register_creature {
        database.validate().unwrap();
    }

    database
}

fn summon_runtime_app(database: ContentDatabase) -> App {
    let mut app = App::new();

    app.insert_resource(database)
        .init_resource::<PendingCombatEffects>()
        .init_resource::<PendingSummons>()
        .init_resource::<CombatEventQueue>()
        .add_systems(Update, super::super::effects::execute_pending_effects);

    app
}

fn enqueue_summon_effect(app: &mut App, source: Option<Entity>, selection: EffectTargetSelection) {
    let database = app.world().resource::<ContentDatabase>().clone();

    app.world_mut()
        .resource_mut::<PendingCombatEffects>()
        .resolve_and_enqueue_from_database(
            &database,
            &effect_id(),
            source,
            selection,
            &CombatAttributes::default(),
        )
        .unwrap();
}

mod runtime_tests;
mod spawn_tests;
mod types_tests;
