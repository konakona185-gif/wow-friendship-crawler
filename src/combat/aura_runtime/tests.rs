use super::*;
use crate::content::{
    AuraRefreshPolicy,
    AuraStackingPolicy,
    AuraStackingScope,
    ContentId,
    ContentMetadata,
    PeriodicAuraEffect,
    PeriodicEffectStackScaling,
};

fn aura_definition(
    stacking_policy: AuraStackingPolicy,
    max_stacks: u32,
) -> AuraDefinition {
    AuraDefinition {
        metadata: ContentMetadata {
            id: aura_id(),
            pack_id: ContentId::new("pack:test").unwrap(),
            name: "Test Aura".to_string(),
            description: String::new(),
            icon: None,
            tags: Vec::new(),
        },
        max_stacks,
        stacking_policy,
        stacking_scope: AuraStackingScope::Shared,
        refresh_policy: AuraRefreshPolicy::ResetDuration,
        periodic_effects: Vec::new(),
    }
}

fn periodic_aura_definition(
    stacking_policy: AuraStackingPolicy,
    max_stacks: u32,
    interval_seconds: f32,
    stack_scaling: PeriodicEffectStackScaling,
) -> AuraDefinition {
    let mut definition = aura_definition(
        stacking_policy,
        max_stacks,
    );

    definition.periodic_effects.push(PeriodicAuraEffect {
        effect: periodic_effect_id(),
        interval_seconds,
        stack_scaling,
    });

    definition
}

fn aura_id() -> ContentId {
    ContentId::new("test:aura.test").unwrap()
}

fn periodic_effect_id() -> ContentId {
    ContentId::new("test:effect.periodic_test").unwrap()
}

fn source() -> Entity {
    Entity::from_raw_u32(1).unwrap()
}

fn other_source() -> Entity {
    Entity::from_raw_u32(2).unwrap()
}

fn third_source() -> Entity {
    Entity::from_raw_u32(3).unwrap()
}

#[test]
fn first_application_creates_an_active_aura() {
    let definition = aura_definition(
        AuraStackingPolicy::AddStacks,
        5,
    );

    let mut active = ActiveAuras::default();

    active
        .apply(&definition, Some(source()), 2, Some(10.0))
        .unwrap();

    assert_eq!(active.len(), 1);
    assert_eq!(active.stacks(&aura_id()), 2);

    let instance = active.get(&aura_id()).unwrap();

    assert_eq!(instance.source, Some(source()));
    assert_eq!(instance.stacks, 2);
    assert_eq!(instance.remaining_duration(), Some(10.0));
}

#[test]
fn add_stacks_increases_stacks_and_refreshes_duration() {
    let definition = aura_definition(
        AuraStackingPolicy::AddStacks,
        5,
    );

    let mut active = ActiveAuras::default();

    active
        .apply(&definition, Some(source()), 2, Some(10.0))
        .unwrap();

    active.advance(3.0);

    active
        .apply(&definition, Some(source()), 2, Some(10.0))
        .unwrap();

    let instance = active.get(&aura_id()).unwrap();

    assert_eq!(instance.stacks, 4);
    assert_eq!(instance.remaining_duration(), Some(10.0));
}

#[test]
fn add_stacks_respects_maximum_stacks() {
    let definition = aura_definition(
        AuraStackingPolicy::AddStacks,
        5,
    );

    let mut active = ActiveAuras::default();

    active
        .apply(&definition, Some(source()), 4, Some(10.0))
        .unwrap();

    active
        .apply(&definition, Some(source()), 4, Some(10.0))
        .unwrap();

    assert_eq!(active.stacks(&aura_id()), 5);
}

#[test]
fn refresh_duration_preserves_existing_stacks() {
    let definition = aura_definition(
        AuraStackingPolicy::RefreshDuration,
        5,
    );

    let mut active = ActiveAuras::default();

    active
        .apply(&definition, Some(source()), 3, Some(10.0))
        .unwrap();

    active.advance(4.0);

    active
        .apply(&definition, Some(source()), 1, Some(8.0))
        .unwrap();

    let instance = active.get(&aura_id()).unwrap();

    assert_eq!(instance.stacks, 3);
    assert_eq!(instance.remaining_duration(), Some(8.0));
}

#[test]
fn replace_policy_overwrites_stacks_and_duration() {
    let definition = aura_definition(
        AuraStackingPolicy::Replace,
        5,
    );

    let mut active = ActiveAuras::default();

    active
        .apply(&definition, Some(source()), 3, Some(10.0))
        .unwrap();

    active.advance(4.0);

    active
        .apply(&definition, Some(source()), 2, Some(8.0))
        .unwrap();

    let instance = active.get(&aura_id()).unwrap();

    assert_eq!(instance.stacks, 2);
    assert_eq!(instance.remaining_duration(), Some(8.0));
}

#[test]
fn independent_policy_creates_separate_instances() {
    let definition = aura_definition(
        AuraStackingPolicy::Independent,
        5,
    );

    let mut active = ActiveAuras::default();

    active
        .apply(&definition, Some(source()), 2, Some(10.0))
        .unwrap();

    active
        .apply(&definition, Some(other_source()), 3, Some(8.0))
        .unwrap();

    assert_eq!(active.len(), 2);
    assert_eq!(active.stacks(&aura_id()), 5);
}

#[test]
fn per_caster_add_stacks_are_isolated_between_sources() {
    let mut definition = aura_definition(
        AuraStackingPolicy::AddStacks,
        5,
    );

    definition.stacking_scope = AuraStackingScope::PerCaster;

    let mut active = ActiveAuras::default();

    active
        .apply(&definition, Some(source()), 2, Some(10.0))
        .unwrap();

    active
        .apply(&definition, Some(other_source()), 3, Some(10.0))
        .unwrap();

    active
        .apply(&definition, Some(source()), 1, Some(10.0))
        .unwrap();

    assert_eq!(active.len(), 2);
    assert_eq!(active.stacks(&aura_id()), 6);

    let first = active
        .iter()
        .find(|instance| instance.source == Some(source()))
        .unwrap();

    let second = active
        .iter()
        .find(|instance| instance.source == Some(other_source()))
        .unwrap();

    assert_eq!(first.stacks, 3);
    assert_eq!(second.stacks, 3);
}

#[test]
fn per_caster_refresh_duration_affects_only_matching_source() {
    let mut definition = aura_definition(
        AuraStackingPolicy::RefreshDuration,
        5,
    );

    definition.stacking_scope = AuraStackingScope::PerCaster;

    let mut active = ActiveAuras::default();

    active
        .apply(&definition, Some(source()), 2, Some(10.0))
        .unwrap();

    active
        .apply(&definition, Some(other_source()), 3, Some(20.0))
        .unwrap();

    active.advance(5.0);

    active
        .apply(&definition, Some(source()), 1, Some(15.0))
        .unwrap();

    let first = active
        .iter()
        .find(|instance| instance.source == Some(source()))
        .unwrap();

    let second = active
        .iter()
        .find(|instance| instance.source == Some(other_source()))
        .unwrap();

    assert_eq!(first.stacks, 2);
    assert_eq!(first.remaining_duration(), Some(15.0));

    assert_eq!(second.stacks, 3);
    assert_eq!(second.remaining_duration(), Some(15.0));
}

#[test]
fn per_caster_replace_affects_only_matching_source() {
    let mut definition = aura_definition(
        AuraStackingPolicy::Replace,
        5,
    );

    definition.stacking_scope = AuraStackingScope::PerCaster;

    let mut active = ActiveAuras::default();

    active
        .apply(&definition, Some(source()), 2, Some(10.0))
        .unwrap();

    active
        .apply(&definition, Some(other_source()), 3, Some(20.0))
        .unwrap();

    active
        .apply(&definition, Some(source()), 4, Some(15.0))
        .unwrap();

    let first = active
        .iter()
        .find(|instance| instance.source == Some(source()))
        .unwrap();

    let second = active
        .iter()
        .find(|instance| instance.source == Some(other_source()))
        .unwrap();

    assert_eq!(first.stacks, 4);
    assert_eq!(first.remaining_duration(), Some(15.0));

    assert_eq!(second.stacks, 3);
    assert_eq!(second.remaining_duration(), Some(20.0));
}

#[test]
fn independent_policy_creates_separate_instances_for_same_caster() {
    let mut definition = aura_definition(
        AuraStackingPolicy::Independent,
        5,
    );

    definition.stacking_scope = AuraStackingScope::PerCaster;

    let mut active = ActiveAuras::default();

    active
        .apply(&definition, Some(source()), 2, Some(10.0))
        .unwrap();

    active
        .apply(&definition, Some(source()), 3, Some(20.0))
        .unwrap();

    assert_eq!(active.len(), 2);
    assert_eq!(active.stacks(&aura_id()), 5);
}

#[test]
fn partial_removal_removes_oldest_stacks_first() {
    let definition = aura_definition(
        AuraStackingPolicy::Independent,
        5,
    );

    let mut active = ActiveAuras::default();

    active
        .apply(&definition, Some(source()), 2, Some(10.0))
        .unwrap();

    active
        .apply(&definition, Some(other_source()), 3, Some(20.0))
        .unwrap();

    assert_eq!(active.remove(&aura_id(), Some(3)), 3);
    assert_eq!(active.stacks(&aura_id()), 2);
    assert_eq!(active.len(), 1);

    let remaining = active.get(&aura_id()).unwrap();

    assert_eq!(remaining.source, Some(other_source()));
    assert_eq!(remaining.stacks, 2);
}

#[test]
fn full_removal_removes_all_matching_instances() {
    let definition = aura_definition(
        AuraStackingPolicy::Independent,
        5,
    );

    let mut active = ActiveAuras::default();

    active
        .apply(&definition, Some(source()), 2, Some(10.0))
        .unwrap();

    active
        .apply(&definition, Some(other_source()), 3, Some(20.0))
        .unwrap();

    assert_eq!(active.remove(&aura_id(), None), 5);
    assert!(active.is_empty());
}

#[test]
fn ticking_expires_timed_auras() {
    let definition = aura_definition(
        AuraStackingPolicy::AddStacks,
        5,
    );

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
fn multiple_periodic_effects_keep_independent_intervals() {
    let mut definition = periodic_aura_definition(
        AuraStackingPolicy::AddStacks,
        5,
        2.0,
        PeriodicEffectStackScaling::Linear,
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
        .apply(&definition, Some(source()), 2, Some(10.0))
        .unwrap();

    let first = active.advance(3.0);

    assert_eq!(first.periodic_ticks.len(), 2);
    assert_eq!(
        first.periodic_ticks[0].effect,
        periodic_effect_id()
    );
    assert_eq!(
        first.periodic_ticks[1].effect,
        second_effect
    );

    let second = active.advance(3.0);

    assert_eq!(second.periodic_ticks.len(), 3);

    assert_eq!(
        second.periodic_ticks
            .iter()
            .filter(|tick| tick.effect == periodic_effect_id())
            .count(),
        2
    );

    assert_eq!(
        second.periodic_ticks
            .iter()
            .filter(|tick| tick.effect == second_effect)
            .count(),
        1
    );

    assert!(
        second.periodic_ticks
            .iter()
            .all(|tick| tick.stacks == 2)
    );
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

#[test]
fn permanent_auras_do_not_expire() {
    let definition = aura_definition(
        AuraStackingPolicy::AddStacks,
        5,
    );

    let mut active = ActiveAuras::default();

    active
        .apply(&definition, Some(source()), 1, None)
        .unwrap();

    assert!(active.tick(1000.0).is_empty());
    assert_eq!(active.stacks(&aura_id()), 1);
}

#[test]
fn invalid_applications_do_not_mutate_active_auras() {
    let definition = aura_definition(
        AuraStackingPolicy::AddStacks,
        5,
    );

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
        active.apply(
            &definition,
            Some(source()),
            1,
            Some(f32::INFINITY)
        ),
        Err(AuraRuntimeError::InvalidDuration)
    );

    assert!(active.is_empty());
}

#[test]
fn invalid_tick_deltas_do_not_change_durations() {
    let definition = aura_definition(
        AuraStackingPolicy::AddStacks,
        5,
    );

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
    let mut definition = aura_definition(
        AuraStackingPolicy::AddStacks,
        5,
    );

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
    let mut definition = aura_definition(
        AuraStackingPolicy::RefreshDuration,
        5,
    );

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

    assert!(
        active.advance(4.0).periodic_ticks.is_empty()
    );

    assert_eq!(
        active.advance(1.0).periodic_ticks.len(),
        1
    );
}

#[test]
fn pandemic_per_caster_refresh_only_preserves_matching_source_duration() {
    let mut definition = aura_definition(
        AuraStackingPolicy::RefreshDuration,
        5,
    );

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
    let mut definition = aura_definition(
        AuraStackingPolicy::Independent,
        5,
    );

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
    let mut definition = aura_definition(
        AuraStackingPolicy::AddStacks,
        5,
    );

    definition.refresh_policy = AuraRefreshPolicy::Pandemic;

    let mut active = ActiveAuras::default();

    active
        .apply(&definition, Some(source()), 2, Some(10.0))
        .unwrap();

    active.advance(5.0);

    let result = active.apply(
        &definition,
        Some(source()),
        0,
        Some(10.0),
    );

    assert_eq!(result, Err(AuraRuntimeError::ZeroStacks));
    assert_eq!(active.stacks(&aura_id()), 2);
    assert_eq!(
        active.get(&aura_id()).unwrap().remaining_duration(),
        Some(5.0)
    );
}
