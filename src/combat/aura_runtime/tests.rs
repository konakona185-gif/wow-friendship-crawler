
use super::*;

use crate::content::{
    AuraRefreshPolicy,
    ContentMetadata,
    PeriodicAuraEffect,
};

fn aura_definition(
    policy: AuraStackingPolicy,
    max_stacks: u32,
) -> AuraDefinition {
    AuraDefinition {
        metadata: ContentMetadata {
            id: ContentId::new("classic:aura.test").unwrap(),
            pack_id: ContentId::new("pack:classic").unwrap(),
            name: "Test Aura".to_string(),
            description: String::new(),
            icon: None,
            tags: Vec::new(),
        },
        max_stacks,
        stacking_policy: policy,
        stacking_scope: AuraStackingScope::Shared,
        refresh_policy: AuraRefreshPolicy::ResetDuration,
        periodic_effects: Vec::new(),
    }
}

fn periodic_aura_definition(
    policy: AuraStackingPolicy,
    max_stacks: u32,
    interval_seconds: f32,
    stack_scaling: PeriodicEffectStackScaling,
) -> AuraDefinition {
    let mut definition = aura_definition(policy, max_stacks);

    definition.periodic_effects.push(PeriodicAuraEffect {
        effect: periodic_effect_id(),
        interval_seconds,
        stack_scaling,
    });

    definition
}

fn aura_id() -> ContentId {
    ContentId::new("classic:aura.test").unwrap()
}

fn periodic_effect_id() -> ContentId {
    ContentId::new("classic:effect.periodic_test").unwrap()
}

fn source() -> Entity {
    Entity::from_bits(1)
}

fn other_source() -> Entity {
    Entity::from_bits(2)
}

fn third_source() -> Entity {
    Entity::from_bits(3)
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

    let aura = active.get(&aura_id()).unwrap();

    assert_eq!(aura.source(), Some(source()));
    assert_eq!(aura.stacks(), 2);
    assert_eq!(aura.remaining_duration(), Some(10.0));
    assert_eq!(active.len(), 1);
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

    active
        .apply(&definition, Some(other_source()), 2, Some(20.0))
        .unwrap();

    let aura = active.get(&aura_id()).unwrap();

    assert_eq!(aura.stacks(), 4);
    assert_eq!(aura.remaining_duration(), Some(20.0));
    assert_eq!(aura.source(), Some(source()));
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

    active
        .apply(&definition, Some(other_source()), 1, Some(25.0))
        .unwrap();

    let aura = active.get(&aura_id()).unwrap();

    assert_eq!(aura.stacks(), 3);
    assert_eq!(aura.remaining_duration(), Some(25.0));
    assert_eq!(aura.source(), Some(source()));
}

#[test]
fn replace_policy_overwrites_stacks_and_duration() {
    let definition = aura_definition(
        AuraStackingPolicy::Replace,
        5,
    );

    let mut active = ActiveAuras::default();

    active
        .apply(&definition, Some(source()), 4, Some(10.0))
        .unwrap();

    active
        .apply(&definition, Some(other_source()), 2, Some(20.0))
        .unwrap();

    let aura = active.get(&aura_id()).unwrap();

    assert_eq!(aura.stacks(), 2);
    assert_eq!(aura.remaining_duration(), Some(20.0));
    assert_eq!(aura.source(), Some(other_source()));
    assert_eq!(active.len(), 1);
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
        .apply(&definition, Some(other_source()), 3, Some(20.0))
        .unwrap();

    assert_eq!(active.len(), 2);
    assert_eq!(active.stacks(&aura_id()), 5);

    let instances: Vec<&ActiveAura> = active.iter().collect();

    assert_eq!(instances[0].source(), Some(source()));
    assert_eq!(instances[0].remaining_duration(), Some(10.0));

    assert_eq!(instances[1].source(), Some(other_source()));
    assert_eq!(instances[1].remaining_duration(), Some(20.0));
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
        .apply(&definition, Some(other_source()), 3, Some(20.0))
        .unwrap();

    active
        .apply(&definition, Some(source()), 1, Some(30.0))
        .unwrap();

    let instances: Vec<&ActiveAura> = active.iter().collect();

    assert_eq!(active.len(), 2);
    assert_eq!(active.stacks(&aura_id()), 6);

    assert_eq!(instances[0].source(), Some(source()));
    assert_eq!(instances[0].stacks(), 3);
    assert_eq!(instances[0].remaining_duration(), Some(30.0));

    assert_eq!(instances[1].source(), Some(other_source()));
    assert_eq!(instances[1].stacks(), 3);
    assert_eq!(instances[1].remaining_duration(), Some(20.0));
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
        .apply(&definition, Some(source()), 3, Some(10.0))
        .unwrap();

    active
        .apply(&definition, Some(other_source()), 2, Some(20.0))
        .unwrap();

    active
        .apply(&definition, Some(source()), 1, Some(30.0))
        .unwrap();

    let instances: Vec<&ActiveAura> = active.iter().collect();

    assert_eq!(active.len(), 2);

    assert_eq!(instances[0].source(), Some(source()));
    assert_eq!(instances[0].stacks(), 3);
    assert_eq!(instances[0].remaining_duration(), Some(30.0));

    assert_eq!(instances[1].source(), Some(other_source()));
    assert_eq!(instances[1].stacks(), 2);
    assert_eq!(instances[1].remaining_duration(), Some(20.0));
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
        .apply(&definition, Some(source()), 4, Some(10.0))
        .unwrap();

    active
        .apply(&definition, Some(other_source()), 2, Some(20.0))
        .unwrap();

    active
        .apply(&definition, Some(third_source()), 1, Some(30.0))
        .unwrap();

    let instances: Vec<&ActiveAura> = active.iter().collect();

    assert_eq!(active.len(), 3);

    assert_eq!(instances[0].source(), Some(source()));
    assert_eq!(instances[0].stacks(), 4);
    assert_eq!(instances[0].remaining_duration(), Some(10.0));

    assert_eq!(instances[1].source(), Some(other_source()));
    assert_eq!(instances[1].stacks(), 2);
    assert_eq!(instances[1].remaining_duration(), Some(20.0));

    assert_eq!(instances[2].source(), Some(third_source()));
    assert_eq!(instances[2].stacks(), 1);
    assert_eq!(instances[2].remaining_duration(), Some(30.0));
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
        .apply(&definition, Some(source()), 1, Some(10.0))
        .unwrap();

    active
        .apply(&definition, Some(source()), 1, Some(20.0))
        .unwrap();

    assert_eq!(active.len(), 2);
    assert_eq!(active.stacks(&aura_id()), 2);
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

    let removed = active.remove(&aura_id(), Some(3));

    assert_eq!(removed, 3);
    assert_eq!(active.len(), 1);
    assert_eq!(active.stacks(&aura_id()), 2);
    assert_eq!(
        active.get(&aura_id()).unwrap().source(),
        Some(other_source())
    );
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

    let removed = active.remove(&aura_id(), None);

    assert_eq!(removed, 5);
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
        .apply(&definition, Some(source()), 2, Some(10.0))
        .unwrap();

    active.advance(8.0);

    active
        .apply(&definition, Some(source()), 1, Some(10.0))
        .unwrap();

    let aura = active.get(&aura_id()).unwrap();

    assert_eq!(aura.stacks(), 3);
    assert_eq!(aura.remaining_duration(), Some(12.0));
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
        .apply(&definition, Some(source()), 3, Some(10.0))
        .unwrap();

    active.advance(1.0);

    active
        .apply(&definition, Some(source()), 1, Some(10.0))
        .unwrap();

    let aura = active.get(&aura_id()).unwrap();

    assert_eq!(aura.stacks(), 3);
    assert_eq!(aura.remaining_duration(), Some(13.0));
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
        .apply(&definition, Some(source()), 4, Some(10.0))
        .unwrap();

    active.advance(4.0);

    active
        .apply(&definition, Some(other_source()), 2, Some(10.0))
        .unwrap();

    let aura = active.get(&aura_id()).unwrap();

    assert_eq!(aura.source(), Some(other_source()));
    assert_eq!(aura.stacks(), 2);
    assert_eq!(aura.remaining_duration(), Some(13.0));

    assert!(
        active.advance(2.0).periodic_ticks.is_empty()
    );

    let result = active.advance(3.0);

    assert_eq!(result.periodic_ticks.len(), 1);
    assert_eq!(result.periodic_ticks[0].source, Some(other_source()));
    assert_eq!(result.periodic_ticks[0].stacks, 2);
}

#[test]
fn pandemic_per_caster_refresh_only_preserves_matching_source_duration() {
    let mut definition = aura_definition(
        AuraStackingPolicy::AddStacks,
        5,
    );

    definition.stacking_scope = AuraStackingScope::PerCaster;
    definition.refresh_policy = AuraRefreshPolicy::Pandemic;

    let mut active = ActiveAuras::default();

    active
        .apply(&definition, Some(source()), 2, Some(10.0))
        .unwrap();

    active
        .apply(&definition, Some(other_source()), 3, Some(14.0))
        .unwrap();

    active.advance(8.0);

    active
        .apply(&definition, Some(source()), 1, Some(10.0))
        .unwrap();

    let instances: Vec<&ActiveAura> = active.iter().collect();

    assert_eq!(instances.len(), 2);

    assert_eq!(instances[0].source(), Some(source()));
    assert_eq!(instances[0].stacks(), 3);
    assert_eq!(instances[0].remaining_duration(), Some(12.0));

    assert_eq!(instances[1].source(), Some(other_source()));
    assert_eq!(instances[1].stacks(), 3);
    assert_eq!(instances[1].remaining_duration(), Some(6.0));
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

    active.advance(8.0);

    active
        .apply(&definition, Some(source()), 1, Some(10.0))
        .unwrap();

    let instances: Vec<&ActiveAura> = active.iter().collect();

    assert_eq!(instances.len(), 2);
    assert_eq!(instances[0].remaining_duration(), Some(2.0));
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
        .apply(&definition, Some(source()), 2, Some(f32::MAX))
        .unwrap();

    let result = active.apply(
        &definition,
        Some(source()),
        1,
        Some(f32::MAX),
    );

    assert_eq!(result, Err(AuraRuntimeError::InvalidDuration));

    let aura = active.get(&aura_id()).unwrap();

    assert_eq!(aura.stacks(), 2);
    assert_eq!(aura.remaining_duration(), Some(f32::MAX));
}
