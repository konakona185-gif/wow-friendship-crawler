use super::*;

#[test]
fn summon_factions_are_opposites() {
    assert_eq!(
        SummonFaction::Allies.opposing(),
        SummonFaction::Enemies
    );

    assert_eq!(
        SummonFaction::Enemies.opposing(),
        SummonFaction::Allies
    );
}

#[test]
fn ally_summon_ai_starts_without_a_target() {
    let ai = AllySummonAi::new(15.0);

    assert_eq!(ai.target, None);
    assert_eq!(ai.aggro_range, 15.0);
}

#[test]
fn summoned_creature_preserves_owner_and_faction() {
    let owner = Entity::from_bits(1);

    let summon = SummonedCreature::new(
        owner,
        creature_id(),
        SummonFaction::Allies,
        Some(30.0),
    );

    assert_eq!(summon.owner, owner);
    assert_eq!(summon.faction, SummonFaction::Allies);
    assert_eq!(summon.creature, creature_id());
    assert_eq!(summon.remaining_duration(), Some(30.0));
}

#[test]
fn permanent_summons_have_no_remaining_duration() {
    let summon = SummonedCreature::new(
        Entity::from_bits(1),
        creature_id(),
        SummonFaction::Enemies,
        None,
    );

    assert_eq!(summon.remaining_duration(), None);
    assert!(!summon.clone().tick_lifetime(1.0));
}

#[test]
fn timed_summon_expires_after_its_duration() {
    let mut summon = SummonedCreature::new(
        Entity::from_bits(1),
        creature_id(),
        SummonFaction::Allies,
        Some(5.0),
    );

    assert!(!summon.tick_lifetime(2.0));
    assert_eq!(summon.remaining_duration(), Some(3.0));

    assert!(!summon.tick_lifetime(2.0));
    assert_eq!(summon.remaining_duration(), Some(1.0));

    assert!(summon.tick_lifetime(1.0));
    assert_eq!(summon.remaining_duration(), Some(0.0));
}

#[test]
fn summon_lifetime_ignores_invalid_delta() {
    let mut summon = SummonedCreature::new(
        Entity::from_bits(1),
        creature_id(),
        SummonFaction::Allies,
        Some(10.0),
    );

    assert!(!summon.tick_lifetime(0.0));
    assert!(!summon.tick_lifetime(f32::NAN));
    assert!(!summon.tick_lifetime(f32::INFINITY));

    assert_eq!(summon.remaining_duration(), Some(10.0));
}
