use bevy::prelude::*;

use crate::content::ContentId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SummonFaction {
    Allies,
    Enemies,
}

impl SummonFaction {
    pub fn opposing(self) -> Self {
        match self {
            Self::Allies => Self::Enemies,
            Self::Enemies => Self::Allies,
        }
    }
}

#[derive(Component, Clone, Debug)]
pub struct SummonedCreature {
    pub owner: Entity,
    pub creature: ContentId,
    pub faction: SummonFaction,
    remaining_duration_seconds: Option<f32>,
}

impl SummonedCreature {
    pub fn new(
        owner: Entity,
        creature: ContentId,
        faction: SummonFaction,
        duration_seconds: Option<f32>,
    ) -> Self {
        Self {
            owner,
            creature,
            faction,
            remaining_duration_seconds: duration_seconds,
        }
    }

    pub fn remaining_duration(&self) -> Option<f32> {
        self.remaining_duration_seconds
    }

    pub(super) fn tick_lifetime(&mut self, delta: f32) -> bool {
        let Some(remaining) = self.remaining_duration_seconds.as_mut() else {
            return false;
        };

        if !delta.is_finite() || delta <= 0.0 {
            return false;
        }

        *remaining = (*remaining - delta).max(0.0);

        *remaining <= 0.0
    }
}

#[derive(Component)]
#[require(
    crate::combat::ActiveAuras,
    crate::combat::PendingDisplacement,
)]
pub struct AllySummon;

#[derive(Clone, Debug)]
pub(super) struct PendingSummon {
    pub owner: Entity,
    pub creature: ContentId,
    pub spawn_position: Vec3,
    pub count: u32,
    pub duration_seconds: Option<f32>,
}

#[derive(Resource, Default)]
pub(super) struct PendingSummons {
    pending: Vec<PendingSummon>,
}

impl PendingSummons {
    pub(super) fn enqueue(&mut self, summon: PendingSummon) {
        self.pending.push(summon);
    }

    pub(super) fn take_pending(&mut self) -> Vec<PendingSummon> {
        std::mem::take(&mut self.pending)
    }

    pub fn len(&self) -> usize {
        self.pending.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn creature_id() -> ContentId {
        ContentId::new("classic_1_12_1:creature.wolf").unwrap()
    }

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

    #[test]
    fn pending_summons_preserve_fifo_order() {
        let mut queue = PendingSummons::default();

        let first = PendingSummon {
            owner: Entity::from_bits(1),
            creature: creature_id(),
            spawn_position: Vec3::ZERO,
            count: 1,
            duration_seconds: None,
        };

        let second = PendingSummon {
            owner: Entity::from_bits(2),
            creature: creature_id(),
            spawn_position: Vec3::X,
            count: 2,
            duration_seconds: Some(10.0),
        };

        queue.enqueue(first.clone());
        queue.enqueue(second.clone());

        let pending = queue.take_pending();

        assert_eq!(pending, vec![first, second]);
        assert!(queue.is_empty());
    }
}
