use super::*;

#[test]
fn aura_application_dispatches_reactive_effect_to_original_applier() {
    let mut app = aura_reactive_runtime_app(EffectTrigger::OnAuraApplied);

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

    let mut subscriptions = CombatEffectSubscriptions::default();

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
        app.world().get::<HealthPool>(applier).unwrap().current(),
        50.0
    );

    assert_eq!(app.world().resource::<PendingCombatEffects>().len(), 1);

    assert!(app.world().resource::<CombatEventQueue>().is_empty());

    app.update();

    assert_eq!(
        app.world().get::<HealthPool>(applier).unwrap().current(),
        60.0
    );

    assert_eq!(
        app.world()
            .get::<CombatStats>(applier)
            .unwrap()
            .current_health,
        60.0
    );

    assert!(app.world().resource::<PendingCombatEffects>().is_empty());
}

#[test]
fn aura_removal_dispatches_reactive_effect_to_remover() {
    let mut app = aura_reactive_runtime_app(EffectTrigger::OnAuraRemoved);

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

    let mut subscriptions = CombatEffectSubscriptions::default();

    subscriptions.subscribe(aura_reactive_effect_id());

    let mut active_auras = ActiveAuras::default();

    active_auras
        .apply(&aura_definition(), Some(remover), 2, Some(20.0))
        .unwrap();

    let aura_owner = app
        .world_mut()
        .spawn((active_auras, subscriptions, CombatAttributes::default()))
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
        app.world().get::<HealthPool>(remover).unwrap().current(),
        50.0
    );

    assert_eq!(app.world().resource::<PendingCombatEffects>().len(), 1);

    app.update();

    assert_eq!(
        app.world().get::<HealthPool>(remover).unwrap().current(),
        60.0
    );

    assert_eq!(
        app.world()
            .get::<CombatStats>(remover)
            .unwrap()
            .current_health,
        60.0
    );

    assert!(app.world().resource::<PendingCombatEffects>().is_empty());
}

#[test]
fn removing_nonexistent_aura_does_not_enqueue_reactive_event() {
    let mut app = runtime_app(true);

    let target = app.world_mut().spawn(ActiveAuras::default()).id();

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

    assert!(app.world().resource::<CombatEventQueue>().is_empty());

    assert!(app.world().resource::<PendingCombatEffects>().is_empty());

    assert!(app.world().get::<ActiveAuras>(target).unwrap().is_empty());
}
