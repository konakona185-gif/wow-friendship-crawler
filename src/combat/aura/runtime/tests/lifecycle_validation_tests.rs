use super::*;

#[test]
fn ticking_expires_timed_auras() {
    let definition = aura_definition(AuraStackingPolicy::AddStacks, 5);

    let mut active = ActiveAuras::default();

    active
        .apply(&definition, Some(source()), 2, Some(5.0))
        .unwrap();

    assert!(active.tick(2.0).is_empty());

    assert_eq!(
        active.get(&aura_id()).unwrap().remaining_duration(),
        Some(3.0)
    );

    let expired = active.tick(3.0);

    assert_eq!(expired.len(), 1);
    assert_eq!(expired[0].aura, aura_id());
    assert_eq!(expired[0].source, Some(source()));
    assert_eq!(expired[0].stacks, 2);
    assert!(active.is_empty());
}

#[test]
fn multiple_periodic_effects_keep_independent_intervals() {
    let mut definition = periodic_aura_definition(
        AuraStackingPolicy::AddStacks,
        5,
        2.0,
        PeriodicEffectStackScaling::Linear,
    );

    let second_effect = ContentId::new("classic:effect.periodic_test_second").unwrap();

    definition.periodic_effects.push(PeriodicAuraEffect {
        effect: second_effect.clone(),
        interval_seconds: 3.0,
        stack_scaling: PeriodicEffectStackScaling::None,
    });

    let mut active = ActiveAuras::default();

    active
        .apply(&definition, Some(source()), 2, Some(10.0))
        .unwrap();

    let first = active.advance(3.0);

    assert_eq!(first.periodic_ticks.len(), 2);
    assert_eq!(first.periodic_ticks[0].effect, periodic_effect_id());
    assert_eq!(first.periodic_ticks[1].effect, second_effect);

    let second = active.advance(3.0);

    assert_eq!(second.periodic_ticks.len(), 3);

    assert_eq!(
        second
            .periodic_ticks
            .iter()
            .filter(|tick| tick.effect == periodic_effect_id())
            .count(),
        2
    );

    assert_eq!(
        second
            .periodic_ticks
            .iter()
            .filter(|tick| tick.effect == second_effect)
            .count(),
        1
    );

    assert!(second.periodic_ticks.iter().all(|tick| tick.stacks == 2));
}

#[test]
fn permanent_auras_do_not_expire() {
    let definition = aura_definition(AuraStackingPolicy::AddStacks, 5);

    let mut active = ActiveAuras::default();

    active.apply(&definition, Some(source()), 1, None).unwrap();

    assert!(active.tick(1000.0).is_empty());
    assert_eq!(active.stacks(&aura_id()), 1);
}

#[test]
fn invalid_applications_do_not_mutate_active_auras() {
    let definition = aura_definition(AuraStackingPolicy::AddStacks, 5);

    let mut active = ActiveAuras::default();

    assert_eq!(
        active.apply(&definition, Some(source()), 0, Some(10.0)),
        Err(AuraRuntimeError::ZeroStacks)
    );

    assert_eq!(
        active.apply(&definition, Some(source()), 1, Some(0.0)),
        Err(AuraRuntimeError::InvalidDuration)
    );

    assert_eq!(
        active.apply(&definition, Some(source()), 1, Some(f32::INFINITY)),
        Err(AuraRuntimeError::InvalidDuration)
    );

    assert!(active.is_empty());
}

#[test]
fn invalid_tick_deltas_do_not_change_durations() {
    let definition = aura_definition(AuraStackingPolicy::AddStacks, 5);

    let mut active = ActiveAuras::default();

    active
        .apply(&definition, Some(source()), 1, Some(10.0))
        .unwrap();

    assert!(active.tick(0.0).is_empty());
    assert!(active.tick(-1.0).is_empty());
    assert!(active.tick(f32::NAN).is_empty());

    assert_eq!(
        active.get(&aura_id()).unwrap().remaining_duration(),
        Some(10.0)
    );
}

#[test]
fn pandemic_add_stacks_preserves_remaining_duration() {
    let mut definition = aura_definition(AuraStackingPolicy::AddStacks, 5);

    definition.refresh_policy = AuraRefreshPolicy::Pandemic;

    let mut active = ActiveAuras::default();

    active
        .apply(&definition, Some(source()), 1, Some(10.0))
        .unwrap();

    active.advance(7.0);

    active
        .apply(&definition, Some(source()), 1, Some(10.0))
        .unwrap();

    let instance = active.get(&aura_id()).unwrap();

    assert_eq!(instance.stacks, 2);
    assert_eq!(instance.remaining_duration(), Some(13.0));
}

#[test]
fn pandemic_refresh_duration_respects_thirty_percent_cap() {
    let mut definition = aura_definition(AuraStackingPolicy::RefreshDuration, 5);

    definition.refresh_policy = AuraRefreshPolicy::Pandemic;

    let mut active = ActiveAuras::default();

    active
        .apply(&definition, Some(source()), 1, Some(10.0))
        .unwrap();

    active.advance(2.0);

    active
        .apply(&definition, Some(source()), 1, Some(10.0))
        .unwrap();

    let instance = active.get(&aura_id()).unwrap();

    assert_eq!(instance.stacks, 1);
    assert_eq!(instance.remaining_duration(), Some(13.0));
}

#[test]
fn pandemic_replace_preserves_duration_and_resets_periodic_progress() {
    let mut definition = periodic_aura_definition(
        AuraStackingPolicy::Replace,
        5,
        5.0,
        PeriodicEffectStackScaling::None,
    );

    definition.refresh_policy = AuraRefreshPolicy::Pandemic;

    let mut active = ActiveAuras::default();

    active
        .apply(&definition, Some(source()), 1, Some(10.0))
        .unwrap();

    active.advance(3.0);

    active
        .apply(&definition, Some(source()), 2, Some(10.0))
        .unwrap();

    let instance = active.get(&aura_id()).unwrap();

    assert_eq!(instance.stacks, 2);
    assert_eq!(instance.remaining_duration(), Some(13.0));

    assert!(active.advance(4.0).periodic_ticks.is_empty());

    assert_eq!(active.advance(1.0).periodic_ticks.len(), 1);
}

#[test]
fn pandemic_per_caster_refresh_only_preserves_matching_source_duration() {
    let mut definition = aura_definition(AuraStackingPolicy::RefreshDuration, 5);

    definition.stacking_scope = AuraStackingScope::PerCaster;
    definition.refresh_policy = AuraRefreshPolicy::Pandemic;

    let mut active = ActiveAuras::default();

    active
        .apply(&definition, Some(source()), 1, Some(10.0))
        .unwrap();

    active
        .apply(&definition, Some(other_source()), 1, Some(20.0))
        .unwrap();

    active.advance(7.0);

    active
        .apply(&definition, Some(source()), 1, Some(10.0))
        .unwrap();

    let first = active
        .iter()
        .find(|instance| instance.source == Some(source()))
        .unwrap();

    let second = active
        .iter()
        .find(|instance| instance.source == Some(other_source()))
        .unwrap();

    assert_eq!(first.remaining_duration(), Some(13.0));
    assert_eq!(second.remaining_duration(), Some(13.0));
}

#[test]
fn pandemic_does_not_carry_duration_between_independent_instances() {
    let mut definition = aura_definition(AuraStackingPolicy::Independent, 5);

    definition.refresh_policy = AuraRefreshPolicy::Pandemic;

    let mut active = ActiveAuras::default();

    active
        .apply(&definition, Some(source()), 1, Some(10.0))
        .unwrap();

    active.advance(7.0);

    active
        .apply(&definition, Some(source()), 1, Some(10.0))
        .unwrap();

    assert_eq!(active.len(), 2);

    let instances: Vec<_> = active.iter().collect();

    assert_eq!(instances[0].remaining_duration(), Some(3.0));
    assert_eq!(instances[1].remaining_duration(), Some(10.0));
}

#[test]
fn failed_pandemic_refresh_does_not_mutate_existing_stacks() {
    let mut definition = aura_definition(AuraStackingPolicy::AddStacks, 5);

    definition.refresh_policy = AuraRefreshPolicy::Pandemic;

    let mut active = ActiveAuras::default();

    active
        .apply(&definition, Some(source()), 2, Some(10.0))
        .unwrap();

    active.advance(5.0);

    let result = active.apply(&definition, Some(source()), 0, Some(10.0));

    assert_eq!(result, Err(AuraRuntimeError::ZeroStacks));
    assert_eq!(active.stacks(&aura_id()), 2);
    assert_eq!(
        active.get(&aura_id()).unwrap().remaining_duration(),
        Some(5.0)
    );
}
