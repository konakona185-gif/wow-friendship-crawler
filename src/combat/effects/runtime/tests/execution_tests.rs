use super::*;

#[test]
fn runtime_executes_damage_and_synchronizes_health_components() {
    let (database, effect_id) = damage_test_database();

    let mut app = App::new();

    let source_entity = app.world_mut().spawn(CombatLevel::new(60).unwrap()).id();

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

        let health = world.get::<HealthPool>(target_entity).unwrap();

        let stats = world.get::<CombatStats>(target_entity).unwrap();

        assert_eq!(health.current(), 60.0);
        assert_eq!(health.maximum(), 100.0);
        assert_eq!(stats.current_health, 60.0);
        assert_eq!(stats.max_health, 100.0);

        assert!(world.resource::<PendingCombatEffects>().is_empty());
    }

    let events = app
        .world_mut()
        .resource_mut::<CombatEventQueue>()
        .drain_with_generation();

    assert_eq!(events.len(), 2);
    assert_eq!(events[0].0.trigger, EffectTrigger::OnDamageDealt);
    assert_eq!(events[1].0.trigger, EffectTrigger::OnDamageTaken);

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
    let request = crate::combat::DamageRequest {
        source: Some(source()),
        target: target(),
        base_amount: 100.0,
        school: content_id("school.physical"),
    };

    let resolution = crate::combat::DamageResolution::new(
        request,
        crate::combat::DamageMitigationResult::new(100.0, 25.0).unwrap(),
    )
    .unwrap();

    let result = EffectExecutionResult::Damage {
        resolution,
        applied_amount: 75.0,
    };

    let effect = event_test_effect();
    let mut event_queue = CombatEventQueue::default();

    enqueue_execution_events(&mut event_queue, &effect, &result, 6);

    let events = event_queue.drain_with_generation();

    assert_eq!(events.len(), 2);
    assert!(events.iter().all(|(_, generation)| *generation == 6));
}

#[test]
fn runtime_does_not_emit_damage_events_when_no_damage_is_applied() {
    let (database, effect_id) = damage_test_database();

    let mut app = App::new();

    let source_entity = app.world_mut().spawn(CombatLevel::new(60).unwrap()).id();

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

    let events = app.world_mut().resource_mut::<CombatEventQueue>().drain();

    assert!(events.is_empty());

    let world = app.world();
    let health = world.get::<HealthPool>(target_entity).unwrap();

    assert_eq!(health.current(), 0.0);
}

#[test]
fn runtime_emits_healing_dealt_and_taken_events() {
    let request = crate::combat::HealingRequest {
        source: Some(source()),
        target: target(),
        base_amount: 25.0,
    };

    let resolution = crate::combat::HealingResolution::new(request, 25.0).unwrap();

    let result = EffectExecutionResult::Healing {
        resolution,
        applied_amount: 25.0,
    };

    let effect = event_test_effect();
    let mut event_queue = CombatEventQueue::default();

    enqueue_execution_events(&mut event_queue, &effect, &result, 0);

    let events = event_queue.drain();

    assert_eq!(events.len(), 2);
    assert_eq!(events[0].trigger, EffectTrigger::OnHealDealt);
    assert_eq!(events[1].trigger, EffectTrigger::OnHealTaken);

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
        (ResourceOperation::Gain, EffectTrigger::OnResourceGained),
        (ResourceOperation::Spend, EffectTrigger::OnResourceSpent),
    ] {
        let request = crate::combat::ResourceRequest {
            source: Some(source()),
            target: target(),
            resource: content_id("resource.mana"),
            operation,
            base_amount: 20.0,
        };

        let resolution = crate::combat::ResourceResolution::new(request, 20.0).unwrap();

        let result = EffectExecutionResult::Resource(resolution);

        let effect = event_test_effect();
        let mut event_queue = CombatEventQueue::default();

        enqueue_execution_events(&mut event_queue, &effect, &result, 0);

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
    let healing_request = crate::combat::HealingRequest {
        source: Some(source()),
        target: target(),
        base_amount: 25.0,
    };

    let healing_resolution = crate::combat::HealingResolution::new(healing_request, 0.0).unwrap();

    let healing_result = EffectExecutionResult::Healing {
        resolution: healing_resolution,
        applied_amount: 0.0,
    };

    let resource_request = crate::combat::ResourceRequest {
        source: Some(source()),
        target: target(),
        resource: content_id("resource.mana"),
        operation: ResourceOperation::Gain,
        base_amount: 20.0,
    };

    let resource_resolution =
        crate::combat::ResourceResolution::new(resource_request, 0.0).unwrap();

    let resource_result = EffectExecutionResult::Resource(resource_resolution);

    let effect = event_test_effect();
    let mut event_queue = CombatEventQueue::default();

    enqueue_execution_events(&mut event_queue, &effect, &healing_result, 0);

    enqueue_execution_events(&mut event_queue, &effect, &resource_result, 0);

    assert!(event_queue.is_empty());
}

#[test]
fn runtime_executes_threat_modification_without_emitting_events() {
    let (database, effect_id) = threat_test_database(35.0);

    let mut app = App::new();

    let source_entity = app.world_mut().spawn_empty().id();

    let target_entity = app.world_mut().spawn(EnemyThreat::default()).id();

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

    let threat = world.get::<EnemyThreat>(target_entity).unwrap();

    assert_eq!(threat.values.get(&source_entity), Some(&35.0));

    assert!(world.resource::<CombatEventQueue>().is_empty());

    assert!(world.resource::<PendingCombatEffects>().is_empty());
}

#[test]
fn runtime_executes_negative_threat_modification_without_emitting_events() {
    let (database, effect_id) = threat_test_database(-20.0);

    let mut app = App::new();

    let source_entity = app.world_mut().spawn_empty().id();

    let mut initial_threat = EnemyThreat::default();
    initial_threat.values.insert(source_entity, 50.0);

    let target_entity = app.world_mut().spawn(initial_threat).id();

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

    let threat = world.get::<EnemyThreat>(target_entity).unwrap();

    assert_eq!(threat.values.get(&source_entity), Some(&30.0));

    assert!(world.resource::<CombatEventQueue>().is_empty());

    assert!(world.resource::<PendingCombatEffects>().is_empty());
}

#[test]
fn generation_limit_is_available_to_runtime() {
    assert_eq!(MAX_REACTIVE_EFFECT_GENERATIONS, 16);
}
