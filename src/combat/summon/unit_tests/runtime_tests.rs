use super::*;

#[test]
fn effect_runtime_enqueues_summon_request() {
    let database = summon_test_database(
        EffectTarget::Caster,
        true,
    );

    let mut app = summon_runtime_app(database);

    let spawn_position = Vec3::new(4.0, 5.0, 6.0);

    let owner = app
        .world_mut()
        .spawn(Transform::from_translation(spawn_position))
        .id();

    enqueue_summon_effect(
        &mut app,
        Some(owner),
        EffectTargetSelection::Caster,
    );

    app.update();

    let pending = app
        .world_mut()
        .resource_mut::<PendingSummons>()
        .take_pending();

    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].owner, owner);
    assert_eq!(pending[0].creature, creature_id());
    assert_eq!(pending[0].spawn_position, spawn_position);
    assert_eq!(pending[0].count, 2);
    assert_eq!(pending[0].duration_seconds, Some(30.0));
}

#[test]
fn effect_runtime_uses_resolved_target_spawn_position() {
    let database = summon_test_database(
        EffectTarget::SelectedTarget,
        true,
    );

    let mut app = summon_runtime_app(database);

    let owner = app
        .world_mut()
        .spawn(Transform::from_xyz(1.0, 2.0, 3.0))
        .id();

    let spawn_position = Vec3::new(8.0, 4.0, -6.0);

    let target = app
        .world_mut()
        .spawn(Transform::from_translation(spawn_position))
        .id();

    enqueue_summon_effect(
        &mut app,
        Some(owner),
        EffectTargetSelection::Selected(target),
    );

    app.update();

    let pending = app
        .world_mut()
        .resource_mut::<PendingSummons>()
        .take_pending();

    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].owner, owner);
    assert_eq!(pending[0].spawn_position, spawn_position);
}

#[test]
fn effect_runtime_rejects_summon_without_source() {
    let database = summon_test_database(
        EffectTarget::SelectedTarget,
        true,
    );

    let mut app = summon_runtime_app(database);

    let target = app
        .world_mut()
        .spawn(Transform::from_xyz(2.0, 0.0, 2.0))
        .id();

    enqueue_summon_effect(
        &mut app,
        None,
        EffectTargetSelection::Selected(target),
    );

    app.update();

    assert!(
        app.world()
            .resource::<PendingSummons>()
            .is_empty()
    );
}

#[test]
fn effect_runtime_rejects_owner_without_transform() {
    let database = summon_test_database(
        EffectTarget::SelectedTarget,
        true,
    );

    let mut app = summon_runtime_app(database);

    let owner = app
        .world_mut()
        .spawn_empty()
        .id();

    let target = app
        .world_mut()
        .spawn(Transform::from_xyz(2.0, 0.0, 2.0))
        .id();

    enqueue_summon_effect(
        &mut app,
        Some(owner),
        EffectTargetSelection::Selected(target),
    );

    app.update();

    assert!(
        app.world()
            .resource::<PendingSummons>()
            .is_empty()
    );
}

#[test]
fn effect_runtime_rejects_target_without_transform() {
    let database = summon_test_database(
        EffectTarget::SelectedTarget,
        true,
    );

    let mut app = summon_runtime_app(database);

    let owner = app
        .world_mut()
        .spawn(Transform::IDENTITY)
        .id();

    let target = app
        .world_mut()
        .spawn_empty()
        .id();

    enqueue_summon_effect(
        &mut app,
        Some(owner),
        EffectTargetSelection::Selected(target),
    );

    app.update();

    assert!(
        app.world()
            .resource::<PendingSummons>()
            .is_empty()
    );
}

#[test]
fn effect_runtime_rejects_unregistered_creature() {
    let database = summon_test_database(
        EffectTarget::Caster,
        false,
    );

    let mut app = summon_runtime_app(database);

    let owner = app
        .world_mut()
        .spawn(Transform::IDENTITY)
        .id();

    enqueue_summon_effect(
        &mut app,
        Some(owner),
        EffectTargetSelection::Caster,
    );

    app.update();

    assert!(
        app.world()
            .resource::<PendingSummons>()
            .is_empty()
    );
}

#[test]
fn effect_runtime_rejects_invalid_summon_parameters() {
    let database = summon_test_database(
        EffectTarget::Caster,
        true,
    );

    let mut app = summon_runtime_app(database);

    let owner = app
        .world_mut()
        .spawn(Transform::IDENTITY)
        .id();

    let mut pending = app
        .world_mut()
        .resource_mut::<PendingCombatEffects>();

    pending.enqueue(ResolvedEffect {
        effect: effect_id(),
        trigger: EffectTrigger::OnCast,
        source: Some(owner),
        target: owner,
        operation: ResolvedEffectOperation::Summon {
            creature: creature_id(),
            count: 0,
            duration_seconds: Some(30.0),
        },
    });

    pending.enqueue(ResolvedEffect {
        effect: effect_id(),
        trigger: EffectTrigger::OnCast,
        source: Some(owner),
        target: owner,
        operation: ResolvedEffectOperation::Summon {
            creature: creature_id(),
            count: 1,
            duration_seconds: Some(f32::NAN),
        },
    });

    drop(pending);

    app.update();

    assert!(
        app.world()
            .resource::<PendingSummons>()
            .is_empty()
    );
}
