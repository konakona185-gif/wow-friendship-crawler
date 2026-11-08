use super::*;

use crate::combat::{
    CombatLevel, CombatStats, DamageRequest, HealthPool, ResolvedEffect, ResolvedEffectOperation,
};

use crate::content::{
    CombatAttributeDefinition, CombatAttributeValueKind, ContentId, ContentMetadata,
    ContentPackManifest, DamageSchoolDefinition, DamageSchoolMitigation, EffectDefinition,
    EffectKind, EffectMagnitude, PeriodicEffectStackScaling,
};

fn source() -> Entity {
    Entity::from_bits(1)
}

fn target() -> Entity {
    Entity::from_bits(2)
}

fn event(
    trigger: EffectTrigger,
    source: Option<Entity>,
    target: Option<Entity>,
) -> CombatEventContext {
    CombatEventContext {
        trigger,
        source,
        target,
        ability: None,
        effect: None,
        outcome: None,
    }
}

fn content_id(value: &str) -> ContentId {
    ContentId::new(format!("dispatcher_tests:{value}")).unwrap()
}

fn classic_content_id(value: &str) -> ContentId {
    ContentId::new(format!("classic_1_12_1:{value}")).unwrap()
}

fn metadata(id: ContentId, pack_id: &ContentId, name: &str) -> ContentMetadata {
    ContentMetadata {
        id,
        pack_id: pack_id.clone(),
        name: name.to_string(),
        description: String::new(),
        icon: None,
        tags: Vec::new(),
    }
}

fn periodic_tick_outcome(
    stacks: u32,
    stack_scaling: PeriodicEffectStackScaling,
) -> CombatEventOutcome {
    CombatEventOutcome::PeriodicTick {
        aura: classic_content_id("aura.periodic_test"),
        stacks,
        stack_scaling,
    }
}

fn test_application(trigger: EffectTrigger, target: EffectTarget) -> (App, ContentId) {
    let pack_id = ContentId::new("pack:dispatcher_tests").unwrap();

    let effect_id = content_id("effect.reactive");

    let mut database = ContentDatabase::new();

    database
        .register_pack(ContentPackManifest {
            id: pack_id.clone(),
            name: "Dispatcher Tests".to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            dependencies: Vec::new(),
        })
        .unwrap();

    database
        .register_effect(EffectDefinition {
            metadata: ContentMetadata {
                id: effect_id.clone(),
                pack_id,
                name: "Reactive Test".to_string(),
                description: String::new(),
                icon: None,
                tags: Vec::new(),
            },
            trigger,
            target,
            kind: EffectKind::Heal {
                amount: EffectMagnitude {
                    base: 10.0,
                    scaling: Vec::new(),
                },
            },
        })
        .unwrap();

    let mut app = App::new();

    app.insert_resource(database)
        .init_resource::<CombatEventQueue>()
        .init_resource::<PendingCombatEffects>()
        .add_systems(Update, dispatch_combat_events);

    (app, effect_id)
}

fn runtime_test_application(trigger: EffectTrigger) -> (App, ContentId) {
    let pack_id = ContentId::new("pack:classic_1_12_1").unwrap();

    let armor_id = ContentId::new("classic_1_12_1:stat.armor").unwrap();

    let school_id = ContentId::new("classic_1_12_1:school.physical").unwrap();

    let effect_id = classic_content_id("effect.reactive");

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
        .register_combat_attribute(CombatAttributeDefinition {
            metadata: metadata(armor_id, &pack_id, "Armor"),
            value_kind: CombatAttributeValueKind::Rating,
        })
        .unwrap();

    database
        .register_damage_school(DamageSchoolDefinition {
            metadata: metadata(school_id.clone(), &pack_id, "Physical"),
            mitigation: DamageSchoolMitigation::Armor,
        })
        .unwrap();

    database
        .register_effect(EffectDefinition {
            metadata: metadata(effect_id.clone(), &pack_id, "Reactive Heal"),
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
        .init_resource::<CombatEventQueue>()
        .init_resource::<PendingCombatEffects>()
        .add_systems(
            Update,
            (
                super::super::runtime::execute_pending_effects,
                dispatch_combat_events,
            )
                .chain(),
        );

    (app, school_id)
}

fn spawn_subscriber(app: &mut App, effect_id: ContentId) -> Entity {
    let mut subscriptions = CombatEffectSubscriptions::default();

    subscriptions.subscribe(effect_id);

    app.world_mut()
        .spawn((subscriptions, CombatAttributes::default()))
        .id()
}

mod area_tests;
mod reactive_runtime_tests;
mod subscriber_tests;
