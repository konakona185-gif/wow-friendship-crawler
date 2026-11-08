use super::*;

#[test]
fn damage_dealt_event_enqueues_matching_reactive_effect() {
    let (mut app, effect_id) = test_application(
        EffectTrigger::OnDamageDealt,
        EffectTarget::SelectedTarget,
    );

    let subscriber = spawn_subscriber(&mut app, effect_id);
    let other = app.world_mut().spawn_empty().id();

    app.world_mut()
        .resource_mut::<CombatEventQueue>()
        .enqueue(event(
            EffectTrigger::OnDamageDealt,
            Some(subscriber),
            Some(other),
        ))
        .unwrap();

    app.update();

    assert_eq!(
        app.world().resource::<PendingCombatEffects>().len(),
        1
    );

    assert!(
        app.world().resource::<CombatEventQueue>().is_empty()
    );
}

#[test]
fn aura_expiration_event_enqueues_matching_reactive_effect() {
    let (mut app, effect_id) = test_application(
        EffectTrigger::OnExpire,
        EffectTarget::SelectedTarget,
    );

    let owner = spawn_subscriber(&mut app, effect_id);
    let applier = app.world_mut().spawn_empty().id();

    app.world_mut()
        .resource_mut::<CombatEventQueue>()
        .enqueue(CombatEventContext {
            trigger: EffectTrigger::OnExpire,
            source: Some(applier),
            target: Some(owner),
            ability: None,
            effect: None,
            outcome: Some(CombatEventOutcome::AuraExpired {
                aura: classic_content_id("aura.test"),
                stacks: 2,
            }),
        })
        .unwrap();

    app.update();

    assert_eq!(
        app.world().resource::<PendingCombatEffects>().len(),
        1
    );

    assert!(
        app.world().resource::<CombatEventQueue>().is_empty()
    );
}

#[test]
fn periodic_tick_event_enqueues_matching_reactive_effect() {
    let (mut app, effect_id) = test_application(
        EffectTrigger::OnPeriodicTick,
        EffectTarget::SelectedTarget,
    );

    let owner = spawn_subscriber(&mut app, effect_id.clone());
    let affected_target = app.world_mut().spawn_empty().id();

    let mut periodic_event = event(
        EffectTrigger::OnPeriodicTick,
        Some(owner),
        Some(affected_target),
    );

    periodic_event.effect = Some(effect_id);
    periodic_event.outcome = Some(periodic_tick_outcome(
        2,
        PeriodicEffectStackScaling::None,
    ));

    app.world_mut()
        .resource_mut::<CombatEventQueue>()
        .enqueue(periodic_event)
        .unwrap();

    app.update();

    assert_eq!(
        app.world().resource::<PendingCombatEffects>().len(),
        1
    );

    assert!(
        app.world().resource::<CombatEventQueue>().is_empty()
    );
}

#[test]
fn periodic_tick_event_does_not_enqueue_another_effect() {
    let (mut app, effect_id) = test_application(
        EffectTrigger::OnPeriodicTick,
        EffectTarget::SelectedTarget,
    );

    let owner = spawn_subscriber(&mut app, effect_id);
    let affected_target = app.world_mut().spawn_empty().id();

    let mut periodic_event = event(
        EffectTrigger::OnPeriodicTick,
        Some(owner),
        Some(affected_target),
    );

    periodic_event.effect = Some(content_id("effect.other"));
    periodic_event.outcome = Some(periodic_tick_outcome(
        2,
        PeriodicEffectStackScaling::Linear,
    ));

    app.world_mut()
        .resource_mut::<CombatEventQueue>()
        .enqueue(periodic_event)
        .unwrap();

    app.update();

    assert_eq!(
        app.world().resource::<PendingCombatEffects>().len(),
        0
    );
}

#[test]
fn periodic_tick_event_without_outcome_is_ignored() {
    let (mut app, effect_id) = test_application(
        EffectTrigger::OnPeriodicTick,
        EffectTarget::SelectedTarget,
    );

    let owner = spawn_subscriber(&mut app, effect_id.clone());
    let affected_target = app.world_mut().spawn_empty().id();

    let mut periodic_event = event(
        EffectTrigger::OnPeriodicTick,
        Some(owner),
        Some(affected_target),
    );

    periodic_event.effect = Some(effect_id);

    app.world_mut()
        .resource_mut::<CombatEventQueue>()
        .enqueue(periodic_event)
        .unwrap();

    app.update();

    assert_eq!(
        app.world().resource::<PendingCombatEffects>().len(),
        0
    );
}

#[test]
fn mismatched_event_trigger_does_not_enqueue_effect() {
    let (mut app, effect_id) = test_application(
        EffectTrigger::OnHealDealt,
        EffectTarget::SelectedTarget,
    );

    let subscriber = spawn_subscriber(&mut app, effect_id);
    let other = app.world_mut().spawn_empty().id();

    app.world_mut()
        .resource_mut::<CombatEventQueue>()
        .enqueue(event(
            EffectTrigger::OnDamageDealt,
            Some(subscriber),
            Some(other),
        ))
        .unwrap();

    app.update();

    assert_eq!(
        app.world().resource::<PendingCombatEffects>().len(),
        0
    );
}

#[test]
fn maximum_generation_does_not_enqueue_reactive_effects() {
    let (mut app, effect_id) = test_application(
        EffectTrigger::OnDamageDealt,
        EffectTarget::SelectedTarget,
    );

    let subscriber = spawn_subscriber(&mut app, effect_id);
    let other = app.world_mut().spawn_empty().id();

    app.world_mut()
        .resource_mut::<CombatEventQueue>()
        .enqueue_with_generation(
            event(
                EffectTrigger::OnDamageDealt,
                Some(subscriber),
                Some(other),
            ),
            MAX_REACTIVE_EFFECT_GENERATIONS,
        )
        .unwrap();

    app.update();

    assert_eq!(
        app.world().resource::<PendingCombatEffects>().len(),
        0
    );
}

#[test]
fn reactive_effect_executes_on_the_following_update() {
    let (mut app, school_id) =
        runtime_test_application(EffectTrigger::OnDamageDealt);

    let effect_id = classic_content_id("effect.reactive");

    let mut subscriptions =
        CombatEffectSubscriptions::default();

    subscriptions.subscribe(effect_id);

    let source = app.world_mut()
        .spawn((
            subscriptions,
            CombatAttributes::default(),
            CombatLevel::new(60).unwrap(),
        ))
        .id();

    let target = app.world_mut()
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

    app.world_mut()
        .resource_mut::<PendingCombatEffects>()
        .enqueue(ResolvedEffect {
            effect: content_id("effect.initial_damage"),
            trigger: EffectTrigger::OnCast,
            source: Some(source),
            target,
            operation: ResolvedEffectOperation::Damage(
                DamageRequest {
                    source: Some(source),
                    target,
                    base_amount: 20.0,
                    school: school_id,
                },
            ),
        });

    app.update();

    assert_eq!(
        app.world()
            .get::<HealthPool>(target)
            .unwrap()
            .current(),
        30.0
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
            .get::<HealthPool>(target)
            .unwrap()
            .current(),
        40.0
    );

    assert_eq!(
        app.world()
            .resource::<PendingCombatEffects>()
            .len(),
        0
    );
}

#[test]
fn periodic_tick_scales_effect_by_stack_count() {
    let (mut app, _) =
        runtime_test_application(EffectTrigger::OnPeriodicTick);

    let effect_id = classic_content_id("effect.reactive");

    let mut subscriptions =
        CombatEffectSubscriptions::default();

    subscriptions.subscribe(effect_id.clone());

    let source = app.world_mut()
        .spawn((
            subscriptions,
            CombatAttributes::default(),
        ))
        .id();

    let target = app.world_mut()
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

    app.world_mut()
        .resource_mut::<CombatEventQueue>()
        .enqueue(CombatEventContext {
            trigger: EffectTrigger::OnPeriodicTick,
            source: Some(source),
            target: Some(target),
            ability: None,
            effect: Some(effect_id),
            outcome: Some(periodic_tick_outcome(
                3,
                PeriodicEffectStackScaling::Linear,
            )),
        })
        .unwrap();

    app.update();

    assert_eq!(
        app.world()
            .get::<HealthPool>(target)
            .unwrap()
            .current(),
        50.0
    );

    assert_eq!(
        app.world().resource::<PendingCombatEffects>().len(),
        1
    );

    app.update();

    assert_eq!(
        app.world()
            .get::<HealthPool>(target)
            .unwrap()
            .current(),
        80.0
    );

    assert_eq!(
        app.world().resource::<PendingCombatEffects>().len(),
        0
    );
}

#[test]
fn aura_application_and_removal_events_use_owner_as_subscriber() {
    for trigger in [
        EffectTrigger::OnAuraApplied,
        EffectTrigger::OnAuraRemoved,
    ] {
        assert_eq!(
            event_participants(&event(
                trigger,
                Some(source()),
                Some(target()),
            )),
            Some((target(), Some(source())))
        );

        assert_eq!(
            event_participants(&event(
                trigger,
                None,
                Some(target()),
            )),
            Some((target(), None))
        );

        assert_eq!(
            event_participants(&event(
                trigger,
                Some(source()),
                None,
            )),
            None
        );
    }
}

#[test]
fn aura_application_and_removal_events_enqueue_reactive_effects() {
    let test_cases = [
        (
            EffectTrigger::OnAuraApplied,
            CombatEventOutcome::AuraApplied {
                aura: classic_content_id("aura.test"),
                total_stacks: 2,
            },
        ),
        (
            EffectTrigger::OnAuraRemoved,
            CombatEventOutcome::AuraRemoved {
                aura: classic_content_id("aura.test"),
                removed_stacks: 1,
            },
        ),
    ];

    for (trigger, outcome) in test_cases {
        let (mut app, effect_id) = test_application(
            trigger,
            EffectTarget::SelectedTarget,
        );

        let owner = spawn_subscriber(&mut app, effect_id);
        let other = app.world_mut().spawn_empty().id();

        let mut aura_event = event(
            trigger,
            Some(other),
            Some(owner),
        );

        aura_event.outcome = Some(outcome);

        app.world_mut()
            .resource_mut::<CombatEventQueue>()
            .enqueue(aura_event)
            .unwrap();

        app.update();

        assert_eq!(
            app.world()
                .resource::<PendingCombatEffects>()
                .len(),
            1,
            "Il trigger {trigger:?} non ha accodato l'effetto reattivo"
        );

        assert!(
            app.world()
                .resource::<CombatEventQueue>()
                .is_empty()
        );
    }
}

#[cfg(test)]
