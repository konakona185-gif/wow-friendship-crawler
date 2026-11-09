use super::*;

#[test]
fn event_context_without_outcome_passes_validation() {
    let context = valid_context(EffectTrigger::OnCast);

    assert!(context.validate().is_ok());
}

#[test]
fn event_context_validates_damage_outcomes() {
    let mut context = valid_context(EffectTrigger::OnDamageDealt);
    context.outcome = Some(damage_outcome());

    assert!(context.validate().is_ok());
}

#[test]
fn event_context_validates_healing_outcomes() {
    let mut context = valid_context(EffectTrigger::OnCast);
    context.outcome = Some(healing_outcome());

    assert!(context.validate().is_ok());
}

#[test]
fn event_context_validates_resource_outcomes() {
    let mut context = valid_context(EffectTrigger::OnCast);
    context.outcome = Some(resource_outcome());

    assert!(context.validate().is_ok());
}

#[test]
fn event_context_validates_aura_application_outcomes() {
    let mut context = valid_context(EffectTrigger::OnAuraApplied);
    context.outcome = Some(aura_applied_outcome(3));

    assert!(context.validate().is_ok());
}

#[test]
fn event_context_validates_aura_removal_outcomes() {
    let mut context = valid_context(EffectTrigger::OnAuraRemoved);
    context.outcome = Some(aura_removed_outcome(2));

    assert!(context.validate().is_ok());
}

#[test]
fn event_context_validates_aura_expiration_outcomes() {
    let mut context = valid_context(EffectTrigger::OnExpire);
    context.outcome = Some(aura_expired_outcome(2));

    assert!(context.validate().is_ok());
}

#[test]
fn event_context_rejects_aura_expiration_with_wrong_trigger() {
    let mut context = valid_context(EffectTrigger::OnCast);
    context.outcome = Some(aura_expired_outcome(1));

    assert_eq!(
        context.validate(),
        Err(CombatEventContextError::AuraExpirationTriggerMismatch)
    );
}

#[test]
fn event_context_validates_periodic_tick_outcomes() {
    let context = valid_periodic_tick_context();

    assert!(context.validate().is_ok());
}

#[test]
fn event_context_rejects_aura_application_with_wrong_trigger() {
    let mut context = valid_context(EffectTrigger::OnCast);
    context.outcome = Some(aura_applied_outcome(1));

    assert_eq!(
        context.validate(),
        Err(CombatEventContextError::AuraApplicationTriggerMismatch)
    );
}

#[test]
fn event_context_rejects_zero_applied_aura_stacks() {
    let mut context = valid_context(EffectTrigger::OnAuraApplied);
    context.outcome = Some(aura_applied_outcome(0));

    assert_eq!(
        context.validate(),
        Err(CombatEventContextError::ZeroAppliedAuraStacks)
    );
}

#[test]
fn event_context_rejects_aura_application_without_target() {
    let mut context = valid_context(EffectTrigger::OnAuraApplied);
    context.target = None;
    context.outcome = Some(aura_applied_outcome(1));

    assert_eq!(
        context.validate(),
        Err(CombatEventContextError::MissingAuraApplicationTarget)
    );
}

#[test]
fn event_context_rejects_aura_removal_with_wrong_trigger() {
    let mut context = valid_context(EffectTrigger::OnCast);
    context.outcome = Some(aura_removed_outcome(1));

    assert_eq!(
        context.validate(),
        Err(CombatEventContextError::AuraRemovalTriggerMismatch)
    );
}

#[test]
fn event_context_rejects_zero_removed_aura_stacks() {
    let mut context = valid_context(EffectTrigger::OnAuraRemoved);
    context.outcome = Some(aura_removed_outcome(0));

    assert_eq!(
        context.validate(),
        Err(CombatEventContextError::ZeroRemovedAuraStacks)
    );
}

#[test]
fn event_context_rejects_aura_removal_without_target() {
    let mut context = valid_context(EffectTrigger::OnAuraRemoved);
    context.target = None;
    context.outcome = Some(aura_removed_outcome(1));

    assert_eq!(
        context.validate(),
        Err(CombatEventContextError::MissingAuraRemovalTarget)
    );
}

#[test]
fn event_context_rejects_zero_expired_aura_stacks() {
    let mut context = valid_context(EffectTrigger::OnExpire);
    context.outcome = Some(aura_expired_outcome(0));

    assert_eq!(
        context.validate(),
        Err(CombatEventContextError::ZeroExpiredAuraStacks)
    );
}

#[test]
fn event_context_rejects_aura_expiration_without_target() {
    let mut context = valid_context(EffectTrigger::OnExpire);
    context.target = None;
    context.outcome = Some(aura_expired_outcome(1));

    assert_eq!(
        context.validate(),
        Err(CombatEventContextError::MissingAuraExpirationTarget)
    );
}

#[test]
fn event_context_rejects_zero_periodic_tick_stacks() {
    let mut context = valid_periodic_tick_context();
    context.outcome = Some(periodic_tick_outcome(0));

    assert_eq!(
        context.validate(),
        Err(CombatEventContextError::ZeroPeriodicTickStacks)
    );
}

#[test]
fn event_context_rejects_periodic_tick_with_wrong_trigger() {
    let mut context = valid_periodic_tick_context();
    context.trigger = EffectTrigger::OnCast;

    assert_eq!(
        context.validate(),
        Err(CombatEventContextError::PeriodicTickTriggerMismatch)
    );
}

#[test]
fn event_context_rejects_periodic_tick_without_target() {
    let mut context = valid_periodic_tick_context();
    context.target = None;

    assert_eq!(
        context.validate(),
        Err(CombatEventContextError::MissingPeriodicTickTarget)
    );
}

#[test]
fn event_context_rejects_periodic_tick_without_effect() {
    let mut context = valid_periodic_tick_context();
    context.effect = None;

    assert_eq!(
        context.validate(),
        Err(CombatEventContextError::MissingPeriodicTickEffect)
    );
}

#[test]
fn event_context_rejects_source_mismatch() {
    let mut context = valid_context(EffectTrigger::OnDamageDealt);

    let mut outcome = damage_outcome();

    if let CombatEventOutcome::Damage(resolution) = &mut outcome {
        resolution.request.source = None;
    }

    context.outcome = Some(outcome);

    assert_eq!(
        context.validate(),
        Err(CombatEventContextError::SourceMismatch)
    );
}

#[test]
fn event_context_rejects_target_mismatch() {
    let mut context = valid_context(EffectTrigger::OnDamageDealt);

    let mut outcome = damage_outcome();

    if let CombatEventOutcome::Damage(resolution) = &mut outcome {
        resolution.request.target = source();
    }

    context.outcome = Some(outcome);

    assert_eq!(
        context.validate(),
        Err(CombatEventContextError::TargetMismatch)
    );
}

#[test]
fn event_context_rejects_invalid_outcomes() {
    let mut context = valid_context(EffectTrigger::OnDamageDealt);

    let mut outcome = damage_outcome();

    if let CombatEventOutcome::Damage(resolution) = &mut outcome {
        resolution.mitigation.mitigated_amount = 150.0;
    }

    context.outcome = Some(outcome);

    assert_eq!(
        context.validate(),
        Err(CombatEventContextError::InvalidDamage(
            DamageResolutionError::InvalidMitigation(
                crate::combat::DamageMitigationError::MitigationExceedsIncomingAmount
            )
        ))
    );
}
