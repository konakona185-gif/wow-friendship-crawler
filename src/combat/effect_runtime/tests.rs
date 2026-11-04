use super::*;

use crate::content::{
    CombatAttributeDefinition,
    CombatAttributeValueKind,
    ContentMetadata,
    ContentPackManifest,
    DamageSchoolDefinition,
    DamageSchoolMitigation,
    EffectDefinition,
    EffectKind,
    EffectMagnitude,
    EffectTarget,
    EffectTrigger,
};

fn content_id(value: &str) -> ContentId {
    ContentId::new(
        format!("classic_1_12_1:{value}"),
    )
    .unwrap()
}

fn test_database(
    target: EffectTarget,
) -> (ContentDatabase, ContentId) {
    let mut database = ContentDatabase::new();

    let pack_id = ContentId::new(
        "pack:classic_1_12_1",
    )
    .unwrap();

    database
        .register_pack(ContentPackManifest {
            id: pack_id.clone(),
            name: "Classic 1.12.1".to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            dependencies: Vec::new(),
        })
        .unwrap();

    let effect_id = content_id("effect.test");

    database
        .register_effect(EffectDefinition {
            metadata: ContentMetadata {
                id: effect_id.clone(),
                pack_id,
                name: "Test Effect".to_string(),
                description: String::new(),
                icon: None,
                tags: Vec::new(),
            },
            trigger: EffectTrigger::OnCast,
            target,
            kind: EffectKind::Interrupt,
        })
        .unwrap();

    (database, effect_id)
}

fn damage_test_database() -> (ContentDatabase, ContentId) {
    let mut database = ContentDatabase::new();

    let pack_id = ContentId::new(
        "pack:classic_1_12_1",
    )
    .unwrap();

    database
        .register_pack(ContentPackManifest {
            id: pack_id.clone(),
            name: "Classic 1.12.1".to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            dependencies: Vec::new(),
        })
        .unwrap();

    let armor_id = content_id("stat.armor");

    database
        .register_combat_attribute(CombatAttributeDefinition {
            metadata: ContentMetadata {
                id: armor_id,
                pack_id: pack_id.clone(),
                name: "Armor".to_string(),
                description: String::new(),
                icon: None,
                tags: Vec::new(),
            },
            value_kind: CombatAttributeValueKind::Rating,
        })
        .unwrap();

    let school_id = content_id("school.physical");

    database
        .register_damage_school(DamageSchoolDefinition {
            metadata: ContentMetadata {
                id: school_id.clone(),
                pack_id: pack_id.clone(),
                name: "Physical".to_string(),
                description: String::new(),
                icon: None,
                tags: Vec::new(),
            },
            mitigation: DamageSchoolMitigation::Armor,
        })
        .unwrap();

    let effect_id = content_id("effect.damage_test");

    database
        .register_effect(EffectDefinition {
            metadata: ContentMetadata {
                id: effect_id.clone(),
                pack_id,
                name: "Damage Test".to_string(),
                description: String::new(),
                icon: None,
                tags: Vec::new(),
            },
            trigger: EffectTrigger::OnCast,
            target: EffectTarget::SelectedTarget,
            kind: EffectKind::Damage {
                amount: EffectMagnitude {
                    base: 40.0,
                    scaling: Vec::new(),
                },
                school: school_id,
            },
        })
        .unwrap();

    database.validate().unwrap();

    (database, effect_id)
}

fn threat_test_database(
    amount: f32,
) -> (ContentDatabase, ContentId) {
    let mut database = ContentDatabase::new();

    let pack_id = ContentId::new(
        "pack:classic_1_12_1",
    )
    .unwrap();

    database
        .register_pack(ContentPackManifest {
            id: pack_id.clone(),
            name: "Classic 1.12.1".to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            dependencies: Vec::new(),
        })
        .unwrap();

    let effect_id = content_id("effect.threat_test");

    database
        .register_effect(EffectDefinition {
            metadata: ContentMetadata {
                id: effect_id.clone(),
                pack_id,
                name: "Threat Test".to_string(),
                description: String::new(),
                icon: None,
                tags: Vec::new(),
            },
            trigger: EffectTrigger::OnCast,
            target: EffectTarget::SelectedTarget,
            kind: EffectKind::ModifyThreat {
                amount: EffectMagnitude {
                    base: amount,
                    scaling: Vec::new(),
                },
            },
        })
        .unwrap();

    database.validate().unwrap();

    (database, effect_id)
}

fn source() -> Entity {
    Entity::from_bits(1)
}

fn target() -> Entity {
    Entity::from_bits(2)
}

fn target_two() -> Entity {
    Entity::from_bits(3)
}

fn event_test_effect() -> ResolvedEffect {
    ResolvedEffect {
        effect: content_id("effect.event_test"),
        trigger: EffectTrigger::OnCast,
        source: Some(source()),
        target: target(),
        operation: super::super::ResolvedEffectOperation::Interrupt,
    }
}

#[test]
fn pending_effects_accept_single_effects() {
    let mut pending = PendingCombatEffects::default();

    pending.enqueue(ResolvedEffect {
        effect: content_id("effect.first"),
        trigger: EffectTrigger::OnCast,
        source: None,
        target: target(),
        operation: super::super::ResolvedEffectOperation::Interrupt,
    });

    assert_eq!(pending.len(), 1);
    assert!(!pending.is_empty());

    assert_eq!(
        pending.pending.front().unwrap().generation,
        0
    );
}

#[test]
fn pending_effects_accept_multiple_effects() {
    let mut pending = PendingCombatEffects::default();

    pending.enqueue_all(vec![
        ResolvedEffect {
            effect: content_id("effect.first"),
            trigger: EffectTrigger::OnCast,
            source: None,
            target: target(),
            operation: super::super::ResolvedEffectOperation::Interrupt,
        },
        ResolvedEffect {
            effect: content_id("effect.second"),
            trigger: EffectTrigger::OnCast,
            source: None,
            target: target_two(),
            operation: super::super::ResolvedEffectOperation::Interrupt,
        },
    ]);

    assert_eq!(pending.len(), 2);

    assert!(
        pending
            .pending
            .iter()
            .all(|effect| effect.generation == 0)
    );
}

#[test]
fn pending_effects_preserve_fifo_order() {
    let mut pending = PendingCombatEffects::default();

    pending.enqueue_all(vec![
        ResolvedEffect {
            effect: content_id("effect.first"),
            trigger: EffectTrigger::OnCast,
            source: None,
            target: target(),
            operation: super::super::ResolvedEffectOperation::Interrupt,
        },
        ResolvedEffect {
            effect: content_id("effect.second"),
            trigger: EffectTrigger::OnCast,
            source: None,
            target: target_two(),
            operation: super::super::ResolvedEffectOperation::Interrupt,
        },
    ]);

    let queued_ids: Vec<ContentId> = pending
        .pending
        .iter()
        .map(|queued| queued.effect.effect.clone())
        .collect();

    assert_eq!(
        queued_ids,
        vec![
            content_id("effect.first"),
            content_id("effect.second"),
        ]
    );
}

#[test]
fn pending_effects_preserve_explicit_generation() {
    let mut pending = PendingCombatEffects::default();

    pending.enqueue_with_generation(
        ResolvedEffect {
            effect: content_id("effect.first"),
            trigger: EffectTrigger::OnCast,
            source: None,
            target: target(),
            operation: super::super::ResolvedEffectOperation::Interrupt,
        },
        5,
    );

    assert_eq!(
        pending.pending.front().unwrap().generation,
        5
    );
}

#[test]
fn pending_effects_apply_generation_to_multiple_effects() {
    let mut pending = PendingCombatEffects::default();

    pending.enqueue_all_with_generation(
        vec![
            ResolvedEffect {
                effect: content_id("effect.first"),
                trigger: EffectTrigger::OnCast,
                source: None,
                target: target(),
                operation: super::super::ResolvedEffectOperation::Interrupt,
            },
            ResolvedEffect {
                effect: content_id("effect.second"),
                trigger: EffectTrigger::OnCast,
                source: None,
                target: target_two(),
                operation: super::super::ResolvedEffectOperation::Interrupt,
            },
        ],
        3,
    );

    assert_eq!(pending.len(), 2);
    assert!(
        pending
            .pending
            .iter()
            .all(|effect| effect.generation == 3)
    );
}

#[test]
fn pending_effects_start_empty() {
    let pending = PendingCombatEffects::default();

    assert!(pending.is_empty());
    assert_eq!(pending.len(), 0);
}

#[test]
fn database_effects_are_resolved_and_enqueued() {
    let (database, effect_id) = test_database(
        EffectTarget::SelectedTarget,
    );

    let mut pending = PendingCombatEffects::default();

    let count = pending
        .resolve_and_enqueue_from_database(
            &database,
            &effect_id,
            Some(source()),
            EffectTargetSelection::Selected(target()),
            &CombatAttributes::default(),
        )
        .unwrap();

    assert_eq!(count, 1);
    assert_eq!(pending.len(), 1);

    let queued = pending.pending.front().unwrap();

    assert_eq!(queued.effect.effect, effect_id);
    assert_eq!(queued.effect.source, Some(source()));
    assert_eq!(queued.effect.target, target());
    assert_eq!(
        queued.effect.operation,
        super::super::ResolvedEffectOperation::Interrupt
    );
    assert_eq!(queued.generation, 0);
}

#[test]
fn database_effects_preserve_requested_generation() {
    let (database, effect_id) = test_database(
        EffectTarget::SelectedTarget,
    );

    let mut pending = PendingCombatEffects::default();

    let count = pending
        .resolve_and_enqueue_from_database_with_generation(
            &database,
            &effect_id,
            Some(source()),
            EffectTargetSelection::Selected(target()),
            &CombatAttributes::default(),
            4,
        )
        .unwrap();

    assert_eq!(count, 1);
    assert_eq!(
        pending.pending.front().unwrap().generation,
        4
    );
}

#[test]
fn area_effects_are_enqueued_in_target_order() {
    let (database, effect_id) = test_database(
        EffectTarget::EnemiesInRadius {
            radius: 10.0,
        },
    );

    let mut pending = PendingCombatEffects::default();

    let count = pending
        .resolve_and_enqueue_from_database(
            &database,
            &effect_id,
            Some(source()),
            EffectTargetSelection::Area(vec![
                target(),
                target_two(),
            ]),
            &CombatAttributes::default(),
        )
        .unwrap();

    assert_eq!(count, 2);
    assert_eq!(pending.len(), 2);

    let targets: Vec<Entity> = pending
        .pending
        .iter()
        .map(|queued| queued.effect.target)
        .collect();

    assert_eq!(
        targets,
        vec![target(), target_two()]
    );
}

#[test]
fn missing_effect_definition_does_not_change_queue() {
    let (database, _) = test_database(
        EffectTarget::SelectedTarget,
    );

    let mut pending = PendingCombatEffects::default();

    pending.enqueue(ResolvedEffect {
        effect: content_id("effect.existing"),
        trigger: EffectTrigger::OnCast,
        source: None,
        target: target(),
        operation: super::super::ResolvedEffectOperation::Interrupt,
    });

    let result = pending.resolve_and_enqueue_from_database(
        &database,
        &content_id("effect.missing"),
        Some(source()),
        EffectTargetSelection::Selected(target_two()),
        &CombatAttributes::default(),
    );

    assert_eq!(
        result,
        Err(EffectQueueError::MissingEffectDefinition(
            content_id("effect.missing")
        ))
    );

    assert_eq!(pending.len(), 1);
}

#[test]
fn failed_resolution_does_not_change_queue() {
    let (database, effect_id) = test_database(
        EffectTarget::SelectedTarget,
    );

    let mut pending = PendingCombatEffects::default();

    let result = pending.resolve_and_enqueue_from_database(
        &database,
        &effect_id,
        Some(source()),
        EffectTargetSelection::Caster,
        &CombatAttributes::default(),
    );

    assert_eq!(
        result,
        Err(EffectQueueError::Resolution(
            EffectResolutionError::TargetSelectionMismatch
        ))
    );

    assert!(pending.is_empty());
}

#[test]
fn runtime_executes_damage_and_synchronizes_health_components() {
    let (database, effect_id) = damage_test_database();

    let mut app = App::new();

    let source_entity = app
        .world_mut()
        .spawn(CombatLevel::new(60).unwrap())
        .id();

    let target_entity = app
        .world_mut()
        .spawn((
            CombatAttributes::default(),
            CombatStats {
                current_health: 100.0,
                max_health: 100.0,
                resource: 0.0,
            },
            HealthPool::new(100.0, 100.0).unwrap(),
        ))
        .id();

    let mut pending = PendingCombatEffects::default();

    let count = pending
        .resolve_and_enqueue_from_database(
            &database,
            &effect_id,
            Some(source_entity),
            EffectTargetSelection::Selected(target_entity),
            &CombatAttributes::default(),
        )
        .unwrap();

    assert_eq!(count, 1);

    app.insert_resource(database)
        .insert_resource(pending)
        .init_resource::<CombatEventQueue>()
        .add_systems(Update, execute_pending_effects);

    app.update();

    {
        let world = app.world();

        let health = world
            .get::<HealthPool>(target_entity)
            .unwrap();

        let stats = world
            .get::<CombatStats>(target_entity)
            .unwrap();

        assert_eq!(health.current(), 60.0);
        assert_eq!(health.maximum(), 100.0);
        assert_eq!(stats.current_health, 60.0);
        assert_eq!(stats.max_health, 100.0);

        assert!(
            world
                .resource::<PendingCombatEffects>()
                .is_empty()
        );
    }

    let events = app
        .world_mut()
        .resource_mut::<CombatEventQueue>()
        .drain_with_generation();

    assert_eq!(events.len(), 2);
    assert_eq!(
        events[0].0.trigger,
        EffectTrigger::OnDamageDealt
    );
    assert_eq!(
        events[1].0.trigger,
        EffectTrigger::OnDamageTaken
    );

    for (event, generation) in &events {
        assert_eq!(event.source, Some(source_entity));
        assert_eq!(event.target, Some(target_entity));
        assert_eq!(event.effect, Some(effect_id.clone()));
        assert_eq!(*generation, 0);
        assert!(matches!(
            &event.outcome,
            Some(CombatEventOutcome::Damage(_))
        ));
    }
}

#[test]
fn runtime_preserves_generation_when_emitting_events() {
    let request = super::super::DamageRequest {
        source: Some(source()),
        target: target(),
        base_amount: 100.0,
        school: content_id("school.physical"),
    };

    let resolution = super::super::DamageResolution::new(
        request,
        super::super::DamageMitigationResult::new(
            100.0,
            25.0,
        )
        .unwrap(),
    )
    .unwrap();

    let result = EffectExecutionResult::Damage {
        resolution,
        applied_amount: 75.0,
    };

    let effect = event_test_effect();
    let mut event_queue = CombatEventQueue::default();

    enqueue_execution_events(
        &mut event_queue,
        &effect,
        &result,
        6,
    );

    let events = event_queue.drain_with_generation();

    assert_eq!(events.len(), 2);
    assert!(
        events
            .iter()
            .all(|(_, generation)| *generation == 6)
    );
}

#[test]
fn runtime_does_not_emit_damage_events_when_no_damage_is_applied() {
    let (database, effect_id) = damage_test_database();

    let mut app = App::new();

    let source_entity = app
        .world_mut()
        .spawn(CombatLevel::new(60).unwrap())
        .id();

    let target_entity = app
        .world_mut()
        .spawn((
            CombatAttributes::default(),
            CombatStats {
                current_health: 0.0,
                max_health: 100.0,
                resource: 0.0,
            },
            HealthPool::new(0.0, 100.0).unwrap(),
        ))
        .id();

    let mut pending = PendingCombatEffects::default();

    pending
        .resolve_and_enqueue_from_database(
            &database,
            &effect_id,
            Some(source_entity),
            EffectTargetSelection::Selected(target_entity),
            &CombatAttributes::default(),
        )
        .unwrap();

    app.insert_resource(database)
        .insert_resource(pending)
        .init_resource::<CombatEventQueue>()
        .add_systems(Update, execute_pending_effects);

    app.update();

    let events = app
        .world_mut()
        .resource_mut::<CombatEventQueue>()
        .drain();

    assert!(events.is_empty());

    let world = app.world();
    let health = world
        .get::<HealthPool>(target_entity)
        .unwrap();

    assert_eq!(health.current(), 0.0);
}

#[test]
fn runtime_emits_healing_dealt_and_taken_events() {
    let request = super::super::HealingRequest {
        source: Some(source()),
        target: target(),
        base_amount: 25.0,
    };

    let resolution = super::super::HealingResolution::new(
        request,
        25.0,
    )
    .unwrap();

    let result = EffectExecutionResult::Healing {
        resolution,
        applied_amount: 25.0,
    };

    let effect = event_test_effect();
    let mut event_queue = CombatEventQueue::default();

    enqueue_execution_events(
        &mut event_queue,
        &effect,
        &result,
        0,
    );

    let events = event_queue.drain();

    assert_eq!(events.len(), 2);
    assert_eq!(
        events[0].trigger,
        EffectTrigger::OnHealDealt
    );
    assert_eq!(
        events[1].trigger,
        EffectTrigger::OnHealTaken
    );

    for event in &events {
        assert_eq!(event.source, Some(source()));
        assert_eq!(event.target, Some(target()));
        assert!(matches!(
            &event.outcome,
            Some(CombatEventOutcome::Healing(_))
        ));
    }
}

#[test]
fn runtime_uses_the_correct_resource_event_trigger() {
    for (operation, expected_trigger) in [
        (
            ResourceOperation::Gain,
            EffectTrigger::OnResourceGained,
        ),
        (
            ResourceOperation::Spend,
            EffectTrigger::OnResourceSpent,
        ),
    ] {
        let request = super::super::ResourceRequest {
            source: Some(source()),
            target: target(),
            resource: content_id("resource.mana"),
            operation,
            base_amount: 20.0,
        };

        let resolution = super::super::ResourceResolution::new(
            request,
            20.0,
        )
        .unwrap();

        let result = EffectExecutionResult::Resource(resolution);

        let effect = event_test_effect();
        let mut event_queue = CombatEventQueue::default();

        enqueue_execution_events(
            &mut event_queue,
            &effect,
            &result,
            0,
        );

        let events = event_queue.drain();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].trigger, expected_trigger);
        assert_eq!(events[0].source, Some(source()));
        assert_eq!(events[0].target, Some(target()));
        assert!(matches!(
            &events[0].outcome,
            Some(CombatEventOutcome::Resource(_))
        ));
    }
}

#[test]
fn runtime_does_not_emit_healing_or_resource_events_for_zero_amounts() {
    let healing_request = super::super::HealingRequest {
        source: Some(source()),
        target: target(),
        base_amount: 25.0,
    };

    let healing_resolution = super::super::HealingResolution::new(
        healing_request,
        0.0,
    )
    .unwrap();

    let healing_result = EffectExecutionResult::Healing {
        resolution: healing_resolution,
        applied_amount: 0.0,
    };

    let resource_request = super::super::ResourceRequest {
        source: Some(source()),
        target: target(),
        resource: content_id("resource.mana"),
        operation: ResourceOperation::Gain,
        base_amount: 20.0,
    };

    let resource_resolution = super::super::ResourceResolution::new(
        resource_request,
        0.0,
    )
    .unwrap();

    let resource_result = EffectExecutionResult::Resource(
        resource_resolution,
    );

    let effect = event_test_effect();
    let mut event_queue = CombatEventQueue::default();

    enqueue_execution_events(
        &mut event_queue,
        &effect,
        &healing_result,
        0,
    );

    enqueue_execution_events(
        &mut event_queue,
        &effect,
        &resource_result,
        0,
    );

    assert!(event_queue.is_empty());
}

#[test]
fn runtime_executes_threat_modification_without_emitting_events() {
    let (database, effect_id) = threat_test_database(35.0);

    let mut app = App::new();

    let source_entity = app
        .world_mut()
        .spawn_empty()
        .id();

    let target_entity = app
        .world_mut()
        .spawn(EnemyThreat::default())
        .id();

    let mut pending = PendingCombatEffects::default();

    let count = pending
        .resolve_and_enqueue_from_database(
            &database,
            &effect_id,
            Some(source_entity),
            EffectTargetSelection::Selected(target_entity),
            &CombatAttributes::default(),
        )
        .unwrap();

    assert_eq!(count, 1);

    app.insert_resource(database)
        .insert_resource(pending)
        .init_resource::<CombatEventQueue>()
        .add_systems(Update, execute_pending_effects);

    app.update();

    let world = app.world();

    let threat = world
        .get::<EnemyThreat>(target_entity)
        .unwrap();

    assert_eq!(
        threat.values.get(&source_entity),
        Some(&35.0)
    );

    assert!(
        world
            .resource::<CombatEventQueue>()
            .is_empty()
    );

    assert!(
        world
            .resource::<PendingCombatEffects>()
            .is_empty()
    );
}

#[test]
fn runtime_executes_negative_threat_modification_without_emitting_events() {
    let (database, effect_id) = threat_test_database(-20.0);

    let mut app = App::new();

    let source_entity = app
        .world_mut()
        .spawn_empty()
        .id();

    let mut initial_threat = EnemyThreat::default();
    initial_threat
        .values
        .insert(source_entity, 50.0);

    let target_entity = app
        .world_mut()
        .spawn(initial_threat)
        .id();

    let mut pending = PendingCombatEffects::default();

    let count = pending
        .resolve_and_enqueue_from_database(
            &database,
            &effect_id,
            Some(source_entity),
            EffectTargetSelection::Selected(target_entity),
            &CombatAttributes::default(),
        )
        .unwrap();

    assert_eq!(count, 1);

    app.insert_resource(database)
        .insert_resource(pending)
        .init_resource::<CombatEventQueue>()
        .add_systems(Update, execute_pending_effects);

    app.update();

    let world = app.world();

    let threat = world
        .get::<EnemyThreat>(target_entity)
        .unwrap();

    assert_eq!(
        threat.values.get(&source_entity),
        Some(&30.0)
    );

    assert!(
        world
            .resource::<CombatEventQueue>()
            .is_empty()
    );

    assert!(
        world
            .resource::<PendingCombatEffects>()
            .is_empty()
    );
}

#[test]
fn generation_limit_is_available_to_runtime() {
    assert_eq!(MAX_REACTIVE_EFFECT_GENERATIONS, 16);
}
