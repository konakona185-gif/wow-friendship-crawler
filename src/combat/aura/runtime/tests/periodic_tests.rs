use super::*;

#[test]
fn periodic_ticks_are_emitted_after_each_interval() {
    let definition = periodic_aura_definition(
        AuraStackingPolicy::AddStacks,
        5,
        2.0,
        PeriodicEffectStackScaling::Linear,
    );

    let mut active = ActiveAuras::default();

    active
        .apply(&definition, Some(source()), 2, Some(10.0))
        .unwrap();

    let first = active.advance(1.5);

    assert!(first.periodic_ticks.is_empty());
    assert!(first.expired.is_empty());

    let second = active.advance(0.5);

    assert_eq!(second.periodic_ticks.len(), 1);

    assert_eq!(
        second.periodic_ticks[0],
        PeriodicAuraTick {
            aura: aura_id(),
            effect: periodic_effect_id(),
            source: Some(source()),
            stacks: 2,
            stack_scaling: PeriodicEffectStackScaling::Linear,
        }
    );

    let third = active.advance(2.0);

    assert_eq!(third.periodic_ticks.len(), 1);
    assert!(third.expired.is_empty());
}


#[test]
fn periodic_effect_catches_up_all_ticks_after_large_delta() {
    let definition = periodic_aura_definition(
        AuraStackingPolicy::AddStacks,
        5,
        2.0,
        PeriodicEffectStackScaling::None,
    );

    let mut active = ActiveAuras::default();

    active
        .apply(&definition, Some(source()), 1, Some(10.0))
        .unwrap();

    let result = active.advance(6.0);

    assert_eq!(result.periodic_ticks.len(), 3);
    assert!(
        result.periodic_ticks
            .iter()
            .all(|tick| tick.effect == periodic_effect_id())
    );

    assert!(result.expired.is_empty());

    let next = active.advance(2.0);

    assert_eq!(next.periodic_ticks.len(), 1);
    assert_eq!(
        next.periodic_ticks[0].effect,
        periodic_effect_id()
    );
}


#[test]
fn periodic_ticks_are_limited_by_aura_duration() {
    let definition = periodic_aura_definition(
        AuraStackingPolicy::AddStacks,
        5,
        2.0,
        PeriodicEffectStackScaling::None,
    );

    let mut active = ActiveAuras::default();

    active
        .apply(&definition, Some(source()), 1, Some(3.0))
        .unwrap();

    let result = active.advance(5.0);

    assert_eq!(result.periodic_ticks.len(), 1);
    assert_eq!(result.expired.len(), 1);
    assert!(active.is_empty());
}


#[test]
fn periodic_tick_aligned_with_expiration_is_emitted() {
    let definition = periodic_aura_definition(
        AuraStackingPolicy::AddStacks,
        5,
        5.0,
        PeriodicEffectStackScaling::None,
    );

    let mut active = ActiveAuras::default();

    active
        .apply(&definition, Some(source()), 1, Some(5.0))
        .unwrap();

    let result = active.advance(5.0);

    assert_eq!(result.periodic_ticks.len(), 1);
    assert_eq!(result.expired.len(), 1);
    assert!(active.is_empty());
}


#[test]
fn adding_stacks_preserves_periodic_tick_progress() {
    let definition = periodic_aura_definition(
        AuraStackingPolicy::AddStacks,
        5,
        5.0,
        PeriodicEffectStackScaling::None,
    );

    let mut active = ActiveAuras::default();

    active
        .apply(&definition, Some(source()), 1, Some(10.0))
        .unwrap();

    assert!(active.advance(3.0).periodic_ticks.is_empty());

    active
        .apply(&definition, Some(source()), 2, Some(10.0))
        .unwrap();

    let result = active.advance(2.0);

    assert_eq!(result.periodic_ticks.len(), 1);
    assert_eq!(result.periodic_ticks[0].stacks, 3);
}


#[test]
fn adding_stacks_preserves_independent_periodic_timers() {
    let mut definition = periodic_aura_definition(
        AuraStackingPolicy::AddStacks,
        5,
        2.0,
        PeriodicEffectStackScaling::None,
    );

    let second_effect = ContentId::new(
        "classic:effect.periodic_test_second",
    )
    .unwrap();

    definition.periodic_effects.push(PeriodicAuraEffect {
        effect: second_effect.clone(),
        interval_seconds: 3.0,
        stack_scaling: PeriodicEffectStackScaling::None,
    });

    let mut active = ActiveAuras::default();

    active
        .apply(&definition, Some(source()), 1, Some(10.0))
        .unwrap();

    assert!(active.advance(1.0).periodic_ticks.is_empty());

    active
        .apply(&definition, Some(source()), 1, Some(10.0))
        .unwrap();

    let first = active.advance(1.0);

    assert_eq!(first.periodic_ticks.len(), 1);
    assert_eq!(
        first.periodic_ticks[0].effect,
        periodic_effect_id()
    );
    assert_eq!(first.periodic_ticks[0].stacks, 2);

    let second = active.advance(1.0);

    assert_eq!(second.periodic_ticks.len(), 1);
    assert_eq!(second.periodic_ticks[0].effect, second_effect);
    assert_eq!(second.periodic_ticks[0].stacks, 2);
}


#[test]
fn refresh_duration_preserves_periodic_timer_progress() {
    let definition = periodic_aura_definition(
        AuraStackingPolicy::RefreshDuration,
        5,
        5.0,
        PeriodicEffectStackScaling::None,
    );

    let mut active = ActiveAuras::default();

    active
        .apply(&definition, Some(source()), 1, Some(10.0))
        .unwrap();

    assert!(active.advance(3.0).periodic_ticks.is_empty());

    active
        .apply(&definition, Some(source()), 1, Some(10.0))
        .unwrap();

    let result = active.advance(2.0);

    assert_eq!(result.periodic_ticks.len(), 1);
    assert_eq!(result.periodic_ticks[0].stacks, 1);
}


#[test]
fn pandemic_refresh_duration_preserves_periodic_timer_progress() {
    let mut definition = periodic_aura_definition(
        AuraStackingPolicy::RefreshDuration,
        5,
        5.0,
        PeriodicEffectStackScaling::None,
    );

    definition.refresh_policy = AuraRefreshPolicy::Pandemic;

    let mut active = ActiveAuras::default();

    active
        .apply(&definition, Some(source()), 1, Some(10.0))
        .unwrap();

    assert!(active.advance(3.0).periodic_ticks.is_empty());

    active
        .apply(&definition, Some(source()), 1, Some(10.0))
        .unwrap();

    let result = active.advance(2.0);

    assert_eq!(result.periodic_ticks.len(), 1);
    assert_eq!(result.periodic_ticks[0].stacks, 1);
}


#[test]
fn per_caster_periodic_timers_advance_independently() {
    let mut definition = periodic_aura_definition(
        AuraStackingPolicy::AddStacks,
        5,
        2.0,
        PeriodicEffectStackScaling::None,
    );

    definition.stacking_scope = AuraStackingScope::PerCaster;

    let mut active = ActiveAuras::default();

    active
        .apply(&definition, Some(source()), 1, Some(10.0))
        .unwrap();

    assert!(active.advance(1.0).periodic_ticks.is_empty());

    active
        .apply(&definition, Some(other_source()), 1, Some(10.0))
        .unwrap();

    let first = active.advance(1.0);

    assert_eq!(first.periodic_ticks.len(), 1);
    assert_eq!(first.periodic_ticks[0].source, Some(source()));

    let second = active.advance(1.0);

    assert_eq!(second.periodic_ticks.len(), 1);
    assert_eq!(second.periodic_ticks[0].source, Some(other_source()));
}


#[test]
fn independent_aura_instances_keep_periodic_timers_separate() {
    let definition = periodic_aura_definition(
        AuraStackingPolicy::Independent,
        5,
        2.0,
        PeriodicEffectStackScaling::None,
    );

    let mut active = ActiveAuras::default();

    active
        .apply(&definition, Some(source()), 1, Some(10.0))
        .unwrap();

    assert!(active.advance(1.0).periodic_ticks.is_empty());

    active
        .apply(&definition, Some(source()), 2, Some(10.0))
        .unwrap();

    let first = active.advance(1.0);

    assert_eq!(first.periodic_ticks.len(), 1);
    assert_eq!(first.periodic_ticks[0].aura, aura_id());
    assert_eq!(first.periodic_ticks[0].stacks, 1);

    let second = active.advance(1.0);

    assert_eq!(second.periodic_ticks.len(), 1);
    assert_eq!(second.periodic_ticks[0].aura, aura_id());
    assert_eq!(second.periodic_ticks[0].stacks, 2);
}


#[test]
fn different_auras_keep_periodic_timers_independent() {
    let first_definition = periodic_aura_definition(
        AuraStackingPolicy::AddStacks,
        5,
        2.0,
        PeriodicEffectStackScaling::None,
    );

    let mut second_definition = periodic_aura_definition(
        AuraStackingPolicy::AddStacks,
        5,
        3.0,
        PeriodicEffectStackScaling::None,
    );

    let second_aura_id = ContentId::new("test:aura.second").unwrap();
    let second_effect_id =
        ContentId::new("test:effect.periodic_second").unwrap();

    second_definition.metadata.id = second_aura_id.clone();
    second_definition.periodic_effects[0].effect =
        second_effect_id.clone();

    let mut active = ActiveAuras::default();

    active
        .apply(&first_definition, Some(source()), 1, Some(10.0))
        .unwrap();

    active
        .apply(&second_definition, Some(source()), 1, Some(10.0))
        .unwrap();

    let first = active.advance(2.0);

    assert_eq!(first.periodic_ticks.len(), 1);
    assert_eq!(first.periodic_ticks[0].aura, aura_id());
    assert_eq!(
        first.periodic_ticks[0].effect,
        periodic_effect_id()
    );

    let second = active.advance(1.0);

    assert_eq!(second.periodic_ticks.len(), 1);
    assert_eq!(second.periodic_ticks[0].aura, second_aura_id);
    assert_eq!(second.periodic_ticks[0].effect, second_effect_id);
}


#[test]
fn partial_stack_removal_updates_future_periodic_tick_stacks() {
    let definition = periodic_aura_definition(
        AuraStackingPolicy::AddStacks,
        5,
        2.0,
        PeriodicEffectStackScaling::Linear,
    );

    let mut active = ActiveAuras::default();

    active
        .apply(&definition, Some(source()), 3, Some(10.0))
        .unwrap();

    assert!(active.advance(1.0).periodic_ticks.is_empty());

    assert_eq!(active.remove(&aura_id(), Some(2)), 2);
    assert_eq!(active.stacks(&aura_id()), 1);

    let result = active.advance(1.0);

    assert_eq!(result.periodic_ticks.len(), 1);
    assert_eq!(result.periodic_ticks[0].stacks, 1);
    assert_eq!(
        result.periodic_ticks[0].stack_scaling,
        PeriodicEffectStackScaling::Linear
    );
}


#[test]
fn removing_aura_stops_future_periodic_ticks() {
    let definition = periodic_aura_definition(
        AuraStackingPolicy::AddStacks,
        5,
        2.0,
        PeriodicEffectStackScaling::None,
    );

    let mut active = ActiveAuras::default();

    active
        .apply(&definition, Some(source()), 1, Some(10.0))
        .unwrap();

    assert!(active.advance(1.0).periodic_ticks.is_empty());

    assert_eq!(active.remove(&aura_id(), None), 1);
    assert!(active.is_empty());

    let result = active.advance(5.0);

    assert!(result.periodic_ticks.is_empty());
    assert!(result.expired.is_empty());
}


#[test]
fn invalid_advance_deltas_do_not_advance_periodic_timers() {
    let definition = periodic_aura_definition(
        AuraStackingPolicy::AddStacks,
        5,
        2.0,
        PeriodicEffectStackScaling::None,
    );

    let mut active = ActiveAuras::default();

    active
        .apply(&definition, Some(source()), 1, Some(10.0))
        .unwrap();

    assert!(active.advance(0.0).periodic_ticks.is_empty());
    assert!(active.advance(-1.0).periodic_ticks.is_empty());
    assert!(active.advance(f32::NAN).periodic_ticks.is_empty());
    assert!(active.advance(f32::INFINITY).periodic_ticks.is_empty());

    let result = active.advance(2.0);

    assert_eq!(result.periodic_ticks.len(), 1);
}


#[test]
fn failed_reapplication_does_not_mutate_existing_aura_or_timer() {
    let definition = periodic_aura_definition(
        AuraStackingPolicy::AddStacks,
        5,
        5.0,
        PeriodicEffectStackScaling::None,
    );

    let mut active = ActiveAuras::default();

    active
        .apply(&definition, Some(source()), 2, Some(10.0))
        .unwrap();

    assert!(active.advance(3.0).periodic_ticks.is_empty());

    assert_eq!(
        active.apply(
            &definition,
            Some(source()),
            2,
            Some(f32::NAN),
        ),
        Err(AuraRuntimeError::InvalidDuration)
    );

    assert_eq!(active.stacks(&aura_id()), 2);
    assert_eq!(
        active.get(&aura_id()).unwrap().remaining_duration(),
        Some(7.0)
    );

    let result = active.advance(2.0);

    assert_eq!(result.periodic_ticks.len(), 1);
    assert_eq!(result.periodic_ticks[0].stacks, 2);
}


#[test]
fn permanent_periodic_auras_continue_emitting_ticks() {
    let definition = periodic_aura_definition(
        AuraStackingPolicy::AddStacks,
        5,
        2.0,
        PeriodicEffectStackScaling::None,
    );

    let mut active = ActiveAuras::default();

    active
        .apply(&definition, Some(source()), 1, None)
        .unwrap();

    let result = active.advance(6.0);

    assert_eq!(result.periodic_ticks.len(), 3);
    assert!(result.expired.is_empty());
    assert_eq!(active.len(), 1);
}


#[test]
fn replacing_aura_restarts_periodic_tick_progress() {
    let definition = periodic_aura_definition(
        AuraStackingPolicy::Replace,
        5,
        5.0,
        PeriodicEffectStackScaling::None,
    );

    let mut active = ActiveAuras::default();

    active
        .apply(&definition, Some(source()), 1, Some(10.0))
        .unwrap();

    assert!(active.advance(3.0).periodic_ticks.is_empty());

    active
        .apply(&definition, Some(other_source()), 2, Some(10.0))
        .unwrap();

    assert!(active.advance(2.0).periodic_ticks.is_empty());

    let result = active.advance(3.0);

    assert_eq!(result.periodic_ticks.len(), 1);
    assert_eq!(result.periodic_ticks[0].source, Some(other_source()));
    assert_eq!(result.periodic_ticks[0].stacks, 2);
}
