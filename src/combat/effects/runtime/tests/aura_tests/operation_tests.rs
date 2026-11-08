use super::*;

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
