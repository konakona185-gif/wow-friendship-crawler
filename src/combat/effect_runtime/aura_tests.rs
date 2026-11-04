
use std::time::Duration;

use bevy::prelude::*;

use super::{
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
                crate::combat::aura_systems::tick_active_auras,
                execute_pending_effects,
                crate::combat::effect_dispatcher::dispatch_combat_events,
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
                crate::combat::aura_systems::tick_active_auras,
                execute_pending_effects,
                crate::combat::effect_dispatcher::dispatch_combat_events,
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

#[test]
fn aura_expiration_dispatches_reactive_effect_to_original_applier() {
    let mut app = expiration_runtime_app();

    let applier = app
        .world_mut()
        .spawn((
            CombatAttributes::default(),
            CombatStats {
                current_health: 50.0,
                max_health: 100.0,
                resource: 0.0,
            },
            HealthPool::new(50.0, 100.0).unwrap(),
        ))
        .id();

    let mut active_auras = ActiveAuras::default();

    active_auras
        .apply(
            &aura_definition(),
            Some(applier),
            1,
            Some(1.0),
        )
        .unwrap();

    let effect_id = ContentId::new(
        "classic_1_12_1:effect.aura_expiration_heal",
    )
    .unwrap();

    let mut subscriptions =
        CombatEffectSubscriptions::default();

    subscriptions.subscribe(effect_id);

    app.world_mut()
        .spawn((
            active_auras,
            subscriptions,
            CombatAttributes::default(),
        ));

    app.update();

    assert_eq!(
        app.world()
            .get::<HealthPool>(applier)
            .unwrap()
            .current(),
        50.0
    );

    assert_eq!(
        app.world()
            .resource::<PendingCombatEffects>()
            .len(),
        1
    );

    assert!(
        app.world()
            .resource::<CombatEventQueue>()
            .is_empty()
    );

    app.update();

    assert_eq!(
        app.world()
            .get::<HealthPool>(applier)
            .unwrap()
            .current(),
        60.0
    );

    assert!(
        app.world()
            .resource::<PendingCombatEffects>()
            .is_empty()
    );
}

#[test]
fn periodic_aura_tick_scales_and_executes_healing() {
    let mut app = periodic_runtime_app();

    let effect_id = periodic_effect_id();

    let mut subscriptions =
        CombatEffectSubscriptions::default();

    subscriptions.subscribe(effect_id);

    let applier = app
        .world_mut()
        .spawn((
            subscriptions,
            CombatAttributes::default(),
        ))
        .id();

    let mut active_auras = ActiveAuras::default();

    active_auras
        .apply(
            &periodic_aura_definition(),
            Some(applier),
            3,
            Some(2.0),
        )
        .unwrap();

    let aura_owner = app
        .world_mut()
        .spawn((
            active_auras,
            CombatAttributes::default(),
            CombatStats {
                current_health: 50.0,
                max_health: 100.0,
                resource: 0.0,
            },
            HealthPool::new(50.0, 100.0).unwrap(),
        ))
        .id();

    app.update();

    assert_eq!(
        app.world()
            .get::<HealthPool>(aura_owner)
            .unwrap()
            .current(),
        50.0
    );

    assert!(
        app.world()
            .get::<ActiveAuras>(aura_owner)
            .unwrap()
            .is_empty()
    );

    assert_eq!(
        app.world()
            .resource::<PendingCombatEffects>()
            .len(),
        1
    );

    assert!(
        app.world()
            .resource::<CombatEventQueue>()
            .is_empty()
    );

    app.update();

    assert_eq!(
        app.world()
            .get::<HealthPool>(aura_owner)
            .unwrap()
            .current(),
        80.0
    );

    assert_eq!(
        app.world()
            .get::<CombatStats>(aura_owner)
            .unwrap()
            .current_health,
        80.0
    );

    assert!(
        app.world()
            .resource::<PendingCombatEffects>()
            .is_empty()
    );
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
                crate::combat::effect_dispatcher::dispatch_combat_events,
            )
                .chain(),
        );

    app
}

#[test]
fn aura_application_dispatches_reactive_effect_to_original_applier() {
    let mut app = aura_reactive_runtime_app(
        EffectTrigger::OnAuraApplied,
    );

    let applier = app
        .world_mut()
        .spawn((
            CombatAttributes::default(),
            CombatStats {
                current_health: 50.0,
                max_health: 100.0,
                resource: 0.0,
            },
            HealthPool::new(50.0, 100.0).unwrap(),
        ))
        .id();

    let mut subscriptions =
        CombatEffectSubscriptions::default();

    subscriptions.subscribe(aura_reactive_effect_id());

    let aura_owner = app
        .world_mut()
        .spawn((
            ActiveAuras::default(),
            subscriptions,
            CombatAttributes::default(),
        ))
        .id();

    enqueue(
        &mut app,
        resolved_aura_effect_from(
            applier,
            aura_owner,
            ResolvedEffectOperation::ApplyAura {
                aura: aura_id(),
                stacks: 2,
                duration_seconds: Some(15.0),
            },
        ),
    );

    app.update();

    assert_eq!(
        app.world()
            .get::<HealthPool>(applier)
            .unwrap()
            .current(),
        50.0
    );

    assert_eq!(
        app.world()
            .resource::<PendingCombatEffects>()
            .len(),
        1
    );

    assert!(
        app.world()
            .resource::<CombatEventQueue>()
            .is_empty()
    );

    app.update();

    assert_eq!(
        app.world()
            .get::<HealthPool>(applier)
            .unwrap()
            .current(),
        60.0
    );

    assert_eq!(
        app.world()
            .get::<CombatStats>(applier)
            .unwrap()
            .current_health,
        60.0
    );

    assert!(
        app.world()
            .resource::<PendingCombatEffects>()
            .is_empty()
    );
}

#[test]
fn aura_removal_dispatches_reactive_effect_to_remover() {
    let mut app = aura_reactive_runtime_app(
        EffectTrigger::OnAuraRemoved,
    );

    let remover = app
        .world_mut()
        .spawn((
            CombatAttributes::default(),
            CombatStats {
                current_health: 50.0,
                max_health: 100.0,
                resource: 0.0,
            },
            HealthPool::new(50.0, 100.0).unwrap(),
        ))
        .id();

    let mut subscriptions =
        CombatEffectSubscriptions::default();

    subscriptions.subscribe(aura_reactive_effect_id());

    let mut active_auras = ActiveAuras::default();

    active_auras
        .apply(
            &aura_definition(),
            Some(remover),
            2,
            Some(20.0),
        )
        .unwrap();

    let aura_owner = app
        .world_mut()
        .spawn((
            active_auras,
            subscriptions,
            CombatAttributes::default(),
        ))
        .id();

    enqueue(
        &mut app,
        resolved_aura_effect_from(
            remover,
            aura_owner,
            ResolvedEffectOperation::RemoveAura {
                aura: aura_id(),
                stacks: Some(1),
            },
        ),
    );

    app.update();

    assert_eq!(
        app.world()
            .get::<ActiveAuras>(aura_owner)
            .unwrap()
            .stacks(&aura_id()),
        1
    );

    assert_eq!(
        app.world()
            .get::<HealthPool>(remover)
            .unwrap()
            .current(),
        50.0
    );

    assert_eq!(
        app.world()
            .resource::<PendingCombatEffects>()
            .len(),
        1
    );

    app.update();

    assert_eq!(
        app.world()
            .get::<HealthPool>(remover)
            .unwrap()
            .current(),
        60.0
    );

    assert_eq!(
        app.world()
            .get::<CombatStats>(remover)
            .unwrap()
            .current_health,
        60.0
    );

    assert!(
        app.world()
            .resource::<PendingCombatEffects>()
            .is_empty()
    );
}

#[test]
fn removing_nonexistent_aura_does_not_enqueue_reactive_event() {
    let mut app = runtime_app(true);

    let target = app
        .world_mut()
        .spawn(ActiveAuras::default())
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

    assert!(
        app.world()
            .resource::<CombatEventQueue>()
            .is_empty()
    );

    assert!(
        app.world()
            .resource::<PendingCombatEffects>()
            .is_empty()
    );

    assert!(
        app.world()
            .get::<ActiveAuras>(target)
            .unwrap()
            .is_empty()
    );
}
