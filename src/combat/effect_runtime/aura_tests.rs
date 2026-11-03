use bevy::prelude::{App, Entity, Update};

use super::{
    execute_pending_effects,
    PendingCombatEffects,
};

use crate::combat::{
    ActiveAuras,
    CombatEventQueue,
    EffectTrigger,
    ResolvedEffect,
    ResolvedEffectOperation,
};

use crate::content::{
    AuraDefinition,
    AuraStackingPolicy,
    ContentDatabase,
    ContentId,
    ContentMetadata,
    ContentPackManifest,
};

fn aura_id() -> ContentId {
    ContentId::new(
        "classic_1_12_1:aura.test",
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
    }
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

#[test]
fn pending_apply_aura_modifies_target_component() {
    let mut app = runtime_app(true);

    let target = app
        .world_mut()
        .spawn(ActiveAuras::default())
        .id();

    enqueue(
        &mut app,
        resolved_aura_effect(
            target,
            ResolvedEffectOperation::ApplyAura {
                aura: aura_id(),
                stacks: 2,
                duration_seconds: Some(15.0),
            },
        ),
    );

    app.update();

    let active = app
        .world()
        .get::<ActiveAuras>(target)
        .unwrap();

    let aura = active.get(&aura_id()).unwrap();

    assert_eq!(aura.stacks(), 2);
    assert_eq!(
        aura.remaining_duration(),
        Some(15.0)
    );
    assert_eq!(aura.source(), Some(source()));

    assert_eq!(
        app.world()
            .resource::<PendingCombatEffects>()
            .len(),
        0
    );
}

#[test]
fn pending_remove_aura_removes_requested_stacks() {
    let mut app = runtime_app(true);

    let mut active = ActiveAuras::default();

    active
        .apply(
            &aura_definition(),
            Some(source()),
            3,
            Some(20.0),
        )
        .unwrap();

    let target = app
        .world_mut()
        .spawn(active)
        .id();

    enqueue(
        &mut app,
        resolved_aura_effect(
            target,
            ResolvedEffectOperation::RemoveAura {
                aura: aura_id(),
                stacks: Some(1),
            },
        ),
    );

    app.update();

    let active = app
        .world()
        .get::<ActiveAuras>(target)
        .unwrap();

    assert_eq!(active.stacks(&aura_id()), 2);
}

#[test]
fn pending_remove_aura_without_stack_limit_removes_all() {
    let mut app = runtime_app(true);

    let mut active = ActiveAuras::default();

    active
        .apply(
            &aura_definition(),
            Some(source()),
            2,
            Some(10.0),
        )
        .unwrap();

    let target = app
        .world_mut()
        .spawn(active)
        .id();

    enqueue(
        &mut app,
        resolved_aura_effect(
            target,
            ResolvedEffectOperation::RemoveAura {
                aura: aura_id(),
                stacks: None,
            },
        ),
    );

    app.update();

    let active = app
        .world()
        .get::<ActiveAuras>(target)
        .unwrap();

    assert!(active.is_empty());
}

#[test]
fn pending_apply_rejects_unregistered_aura() {
    let mut app = runtime_app(false);

    let target = app
        .world_mut()
        .spawn(ActiveAuras::default())
        .id();

    enqueue(
        &mut app,
        resolved_aura_effect(
            target,
            ResolvedEffectOperation::ApplyAura {
                aura: aura_id(),
                stacks: 1,
                duration_seconds: Some(10.0),
            },
        ),
    );

    app.update();

    let active = app
        .world()
        .get::<ActiveAuras>(target)
        .unwrap();

    assert!(active.is_empty());
}

#[test]
fn pending_apply_handles_missing_target_component() {
    let mut app = runtime_app(true);

    let target = app
        .world_mut()
        .spawn_empty()
        .id();

    enqueue(
        &mut app,
        resolved_aura_effect(
            target,
            ResolvedEffectOperation::ApplyAura {
                aura: aura_id(),
                stacks: 1,
                duration_seconds: Some(10.0),
            },
        ),
    );

    app.update();

    assert!(
        app.world()
            .get::<ActiveAuras>(target)
            .is_none()
    );

    assert!(
        app.world()
            .resource::<PendingCombatEffects>()
            .is_empty()
    );
}
