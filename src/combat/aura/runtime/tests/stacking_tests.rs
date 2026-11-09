use super::*;

#[test]
fn first_application_creates_an_active_aura() {
    let definition = aura_definition(AuraStackingPolicy::AddStacks, 5);

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
    let definition = aura_definition(AuraStackingPolicy::AddStacks, 5);

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
    let definition = aura_definition(AuraStackingPolicy::AddStacks, 5);

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
    let definition = aura_definition(AuraStackingPolicy::RefreshDuration, 5);

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
    let definition = aura_definition(AuraStackingPolicy::Replace, 5);

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
    let definition = aura_definition(AuraStackingPolicy::Independent, 5);

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
    let mut definition = aura_definition(AuraStackingPolicy::AddStacks, 5);

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
    let mut definition = aura_definition(AuraStackingPolicy::RefreshDuration, 5);

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
    let mut definition = aura_definition(AuraStackingPolicy::Replace, 5);

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
    let mut definition = aura_definition(AuraStackingPolicy::Independent, 5);

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
    let definition = aura_definition(AuraStackingPolicy::Independent, 5);

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
    let definition = aura_definition(AuraStackingPolicy::Independent, 5);

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
