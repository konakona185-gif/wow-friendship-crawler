use super::*;

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
