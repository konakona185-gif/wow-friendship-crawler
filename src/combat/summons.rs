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

#[derive(Clone, Debug, PartialEq)]
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

    use crate::combat::{
        CombatAttributes,
        CombatEventQueue,
        EffectTargetSelection,
        ResolvedEffect,
        ResolvedEffectOperation,
    };

    use crate::content::{
        ContentDatabase,
        ContentMetadata,
        ContentPackManifest,
        CreatureDefinition,
        EffectDefinition,
        EffectKind,
        EffectTarget,
        EffectTrigger,
    };

    use super::super::effect_runtime::PendingCombatEffects;

    fn creature_id() -> ContentId {
        ContentId::new("classic_1_12_1:creature.imp").unwrap()
    }

    fn effect_id() -> ContentId {
        ContentId::new("classic_1_12_1:effect.summon_imp").unwrap()
    }

    fn pack_id() -> ContentId {
        ContentId::new("pack:classic_1_12_1").unwrap()
    }

    fn summon_test_database(
        target: EffectTarget,
        register_creature: bool,
    ) -> ContentDatabase {
        let mut database = ContentDatabase::new();

        let pack_id = pack_id();
        let creature_id = creature_id();
        let effect_id = effect_id();

        database
            .register_pack(ContentPackManifest {
                id: pack_id.clone(),
                name: "Classic 1.12.1".to_string(),
                version: "1.0.0".to_string(),
                description: String::new(),
                dependencies: Vec::new(),
            })
            .unwrap();

        if register_creature {
            database
                .register_creature(CreatureDefinition {
                    metadata: ContentMetadata {
                        id: creature_id.clone(),
                        pack_id: pack_id.clone(),
                        name: "Imp".to_string(),
                        description: String::new(),
                        icon: None,
                        tags: vec!["summon".to_string()],
                    },
                    health: 150.0,
                    attack_damage: 12.0,
                    attack_interval_seconds: 2.0,
                    melee_range: 2.5,
                    aggro_range: 15.0,
                    leash_range: 40.0,
                    movement_speed: 5.0,
                })
                .unwrap();
        }

        database
            .register_effect(EffectDefinition {
                metadata: ContentMetadata {
                    id: effect_id,
                    pack_id,
                    name: "Summon Imp".to_string(),
                    description: String::new(),
                    icon: None,
                    tags: vec!["summon".to_string()],
                },
                trigger: EffectTrigger::OnCast,
                target,
                kind: EffectKind::Summon {
                    creature: creature_id,
                    count: 2,
                    duration_seconds: Some(30.0),
                },
            })
            .unwrap();

        if register_creature {
            database.validate().unwrap();
        }

        database
    }

    fn summon_runtime_app(database: ContentDatabase) -> App {
        let mut app = App::new();

        app.insert_resource(database)
            .init_resource::<PendingCombatEffects>()
            .init_resource::<PendingSummons>()
            .init_resource::<CombatEventQueue>()
            .add_systems(
                Update,
                super::super::effect_runtime::execute_pending_effects,
            );

        app
    }

    fn enqueue_summon_effect(
        app: &mut App,
        source: Option<Entity>,
        selection: EffectTargetSelection,
    ) {
        let database = app
            .world()
            .resource::<ContentDatabase>()
            .clone();

        app.world_mut()
            .resource_mut::<PendingCombatEffects>()
            .resolve_and_enqueue_from_database(
                &database,
                &effect_id(),
                source,
                selection,
                &CombatAttributes::default(),
            )
            .unwrap();
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
}
