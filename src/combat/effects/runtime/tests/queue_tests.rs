use super::*;

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

    assert_eq!(pending.pending.front().unwrap().generation, 0);
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

    assert!(pending.pending.iter().all(|effect| effect.generation == 0));
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
        vec![content_id("effect.first"), content_id("effect.second"),]
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

    assert_eq!(pending.pending.front().unwrap().generation, 5);
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
    assert!(pending.pending.iter().all(|effect| effect.generation == 3));
}

#[test]
fn pending_effects_start_empty() {
    let pending = PendingCombatEffects::default();

    assert!(pending.is_empty());
    assert_eq!(pending.len(), 0);
}

#[test]
fn database_effects_are_resolved_and_enqueued() {
    let (database, effect_id) = test_database(EffectTarget::SelectedTarget);

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
    let (database, effect_id) = test_database(EffectTarget::SelectedTarget);

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
    assert_eq!(pending.pending.front().unwrap().generation, 4);
}

#[test]
fn area_effects_are_enqueued_in_target_order() {
    let (database, effect_id) = test_database(EffectTarget::EnemiesInRadius { radius: 10.0 });

    let mut pending = PendingCombatEffects::default();

    let count = pending
        .resolve_and_enqueue_from_database(
            &database,
            &effect_id,
            Some(source()),
            EffectTargetSelection::Area(vec![target(), target_two()]),
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

    assert_eq!(targets, vec![target(), target_two()]);
}

#[test]
fn missing_effect_definition_does_not_change_queue() {
    let (database, _) = test_database(EffectTarget::SelectedTarget);

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
        Err(EffectQueueError::MissingEffectDefinition(content_id(
            "effect.missing"
        )))
    );

    assert_eq!(pending.len(), 1);
}

#[test]
fn failed_resolution_does_not_change_queue() {
    let (database, effect_id) = test_database(EffectTarget::SelectedTarget);

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
