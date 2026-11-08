
use std::time::Duration;

use bevy::prelude::*;

use super::super::{
    execute_pending_effects,
    PendingCombatEffects,
};

use crate::combat::{
    ActiveAuras,
    CombatAttributes,
    CombatEffectSubscriptions,
    CombatEventQueue,
    CombatStats,
    EffectTrigger,
    HealthPool,
    ResolvedEffect,
    ResolvedEffectOperation,
};

use crate::content::{
    AuraDefinition,
    AuraRefreshPolicy,
    AuraStackingPolicy,
    AuraStackingScope,
    ContentDatabase,
    ContentId,
    ContentMetadata,
    ContentPackManifest,
    EffectDefinition,
    EffectKind,
    EffectMagnitude,
    EffectTarget,
    PeriodicAuraEffect,
    PeriodicEffectStackScaling,
};

fn aura_id() -> ContentId {
    ContentId::new(
        "classic_1_12_1:aura.test",
    )
    .unwrap()
}

fn periodic_effect_id() -> ContentId {
    ContentId::new(
        "classic_1_12_1:effect.periodic_test",
    )
    .unwrap()
}

fn source() -> Entity {
    Entity::from_bits(1)
}

fn aura_definition() -> AuraDefinition {
    AuraDefinition {
        metadata: ContentMetadata {
            id: aura_id(),
            pack_id: ContentId::new(
                "pack:classic_1_12_1",
            )
            .unwrap(),
            name: "Test Aura".to_string(),
            description: String::new(),
            icon: None,
            tags: Vec::new(),
        },
        max_stacks: 5,
        stacking_policy: AuraStackingPolicy::AddStacks,
        stacking_scope: AuraStackingScope::Shared,
        refresh_policy: AuraRefreshPolicy::ResetDuration,
        periodic_effects: Vec::new(),
    }
}

fn periodic_aura_definition() -> AuraDefinition {
    let mut definition = aura_definition();

    definition.periodic_effects.push(PeriodicAuraEffect {
        effect: periodic_effect_id(),
        interval_seconds: 2.0,
        stack_scaling: PeriodicEffectStackScaling::Linear,
    });

    definition
}

fn runtime_app(register_aura: bool) -> App {
    let pack_id = ContentId::new(
        "pack:classic_1_12_1",
    )
    .unwrap();

    let mut database = ContentDatabase::new();

    database
        .register_pack(ContentPackManifest {
            id: pack_id,
            name: "Classic 1.12.1".to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            dependencies: Vec::new(),
        })
        .unwrap();

    if register_aura {
        database
            .register_aura(aura_definition())
            .unwrap();
    }

    let mut app = App::new();

    app.insert_resource(database)
        .init_resource::<PendingCombatEffects>()
        .init_resource::<CombatEventQueue>()
        .add_systems(Update, execute_pending_effects);

    app
}

fn expiration_runtime_app() -> App {
    let pack_id = ContentId::new(
        "pack:classic_1_12_1",
    )
    .unwrap();

    let effect_id = ContentId::new(
        "classic_1_12_1:effect.aura_expiration_heal",
    )
    .unwrap();

    let mut database = ContentDatabase::new();

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
        .register_aura(aura_definition())
        .unwrap();

    database
        .register_effect(EffectDefinition {
            metadata: ContentMetadata {
                id: effect_id,
                pack_id,
                name: "Aura Expiration Heal".to_string(),
                description: String::new(),
                icon: None,
                tags: Vec::new(),
            },
            trigger: EffectTrigger::OnExpire,
            target: EffectTarget::SelectedTarget,
            kind: EffectKind::Heal {
                amount: EffectMagnitude {
                    base: 10.0,
                    scaling: Vec::new(),
                },
            },
        })
        .unwrap();

    database.validate().unwrap();

    let mut time = Time::<()>::default();
    time.advance_by(Duration::from_secs(2));

    let mut app = App::new();

    app.insert_resource(database)
        .insert_resource(time)
        .init_resource::<PendingCombatEffects>()
        .init_resource::<CombatEventQueue>()
        .add_systems(
            Update,
            (
                crate::combat::aura::tick_active_auras,
                execute_pending_effects,
                crate::combat::effects::dispatch_combat_events,
            )
                .chain(),
        );

    app
}

fn periodic_runtime_app() -> App {
    let pack_id = ContentId::new(
        "pack:classic_1_12_1",
    )
    .unwrap();

    let mut database = ContentDatabase::new();

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
        .register_aura(periodic_aura_definition())
        .unwrap();

    database
        .register_effect(EffectDefinition {
            metadata: ContentMetadata {
                id: periodic_effect_id(),
                pack_id,
                name: "Periodic Test Heal".to_string(),
                description: String::new(),
                icon: None,
                tags: Vec::new(),
            },
            trigger: EffectTrigger::OnPeriodicTick,
            target: EffectTarget::SelectedTarget,
            kind: EffectKind::Heal {
                amount: EffectMagnitude {
                    base: 10.0,
                    scaling: Vec::new(),
                },
            },
        })
        .unwrap();

    database.validate().unwrap();

    let mut time = Time::<()>::default();
    time.advance_by(Duration::from_secs(2));

    let mut app = App::new();

    app.insert_resource(database)
        .insert_resource(time)
        .init_resource::<PendingCombatEffects>()
        .init_resource::<CombatEventQueue>()
        .add_systems(
            Update,
            (
                crate::combat::aura::tick_active_auras,
                execute_pending_effects,
                crate::combat::effects::dispatch_combat_events,
            )
                .chain(),
        );

    app
}

fn resolved_aura_effect(
    target: Entity,
    operation: ResolvedEffectOperation,
) -> ResolvedEffect {
    ResolvedEffect {
        effect: ContentId::new(
            "classic_1_12_1:effect.aura_test",
        )
        .unwrap(),
        trigger: EffectTrigger::OnCast,
        source: Some(source()),
        target,
        operation,
    }
}

fn enqueue(
    app: &mut App,
    effect: ResolvedEffect,
) {
    app.world_mut()
        .resource_mut::<PendingCombatEffects>()
        .enqueue(effect);
}

fn aura_reactive_effect_id() -> ContentId {
    ContentId::new(
        "classic_1_12_1:effect.aura_reactive",
    )
    .unwrap()
}

fn resolved_aura_effect_from(
    source_entity: Entity,
    target: Entity,
    operation: ResolvedEffectOperation,
) -> ResolvedEffect {
    ResolvedEffect {
        effect: ContentId::new(
            "classic_1_12_1:effect.aura_test",
        )
        .unwrap(),
        trigger: EffectTrigger::OnCast,
        source: Some(source_entity),
        target,
        operation,
    }
}

fn aura_reactive_runtime_app(
    trigger: EffectTrigger,
) -> App {
    let pack_id = ContentId::new(
        "pack:classic_1_12_1",
    )
    .unwrap();

    let effect_id = aura_reactive_effect_id();

    let mut database = ContentDatabase::new();

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
        .register_aura(aura_definition())
        .unwrap();

    database
        .register_effect(EffectDefinition {
            metadata: ContentMetadata {
                id: effect_id,
                pack_id,
                name: "Aura Reactive Heal".to_string(),
                description: String::new(),
                icon: None,
                tags: Vec::new(),
            },
            trigger,
            target: EffectTarget::SelectedTarget,
            kind: EffectKind::Heal {
                amount: EffectMagnitude {
                    base: 10.0,
                    scaling: Vec::new(),
                },
            },
        })
        .unwrap();

    database.validate().unwrap();

    let mut app = App::new();

    app.insert_resource(database)
        .init_resource::<PendingCombatEffects>()
        .init_resource::<CombatEventQueue>()
        .add_systems(
            Update,
            (
                execute_pending_effects,
                crate::combat::effects::dispatch_combat_events,
            )
                .chain(),
        );

    app
}

mod reactive_tests;
mod lifecycle_tests;
mod operation_tests;
