use bevy::prelude::*;

use crate::content::{ContentDatabase, ContentId};
use crate::player::Character;

use super::{
    CombatAttributes,
    CombatLevel,
    CombatStats,
    Enemy,
    EnemyAi,
    EnemyHomePosition,
    HealthPool,
};

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

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct AllySummonAi {
    pub target: Option<Entity>,
    pub aggro_range: f32,
}

impl AllySummonAi {
    pub fn new(aggro_range: f32) -> Self {
        Self {
            target: None,
            aggro_range,
        }
    }
}

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

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub(crate) struct SummonCombatProfile {
    pub attack_damage: f32,
    pub attack_interval_seconds: f32,
    pub melee_range: f32,
    pub aggro_range: f32,
    pub leash_range: f32,
    pub movement_speed: f32,
}

pub(super) fn spawn_pending_summons(
    mut commands: Commands,
    mut pending: ResMut<PendingSummons>,
    database: Res<ContentDatabase>,
    owners: Query<(
        &CombatLevel,
        Has<Character>,
        Has<Enemy>,
        Option<&SummonedCreature>,
    )>,
) {
    let requests = pending.take_pending();

    for request in requests {
        if request.count == 0
            || !request.spawn_position.is_finite()
            || request.duration_seconds.is_some_and(|duration| {
                !duration.is_finite() || duration <= 0.0
            })
        {
            warn!(
                owner = ?request.owner,
                creature = %request.creature.as_str(),
                count = request.count,
                duration_seconds = ?request.duration_seconds,
                spawn_position = ?request.spawn_position,
                "Impossibile creare l'evocazione: richiesta non valida."
            );
            continue;
        }

        let Ok((
            owner_level,
            is_character,
            is_enemy,
            parent_summon,
        )) = owners.get(request.owner)
        else {
            warn!(
                owner = ?request.owner,
                creature = %request.creature.as_str(),
                "Impossibile creare l'evocazione: proprietario mancante o privo di CombatLevel."
            );
            continue;
        };

        let faction = if let Some(parent_summon) = parent_summon {
            parent_summon.faction
        } else if is_character && !is_enemy {
            SummonFaction::Allies
        } else if is_enemy && !is_character {
            SummonFaction::Enemies
        } else {
            warn!(
                owner = ?request.owner,
                creature = %request.creature.as_str(),
                "Impossibile determinare la fazione runtime del proprietario."
            );
            continue;
        };

        let Some(creature) = database.creature(&request.creature) else {
            warn!(
                owner = ?request.owner,
                creature = %request.creature.as_str(),
                "Impossibile creare l'evocazione: creatura non registrata."
            );
            continue;
        };

        let health_pool = match HealthPool::new(
            creature.health,
            creature.health,
        ) {
            Ok(pool) => pool,
            Err(error) => {
                warn!(
                    creature = %request.creature.as_str(),
                    %error,
                    "Impossibile inizializzare la salute dell'evocazione."
                );
                continue;
            }
        };

        let level = *owner_level;

        let combat_profile = SummonCombatProfile {
            attack_damage: creature.attack_damage,
            attack_interval_seconds: creature.attack_interval_seconds,
            melee_range: creature.melee_range,
            aggro_range: creature.aggro_range,
            leash_range: creature.leash_range,
            movement_speed: creature.movement_speed,
        };

        for _ in 0..request.count {
            let mut entity_commands = commands.spawn((
                SummonedCreature::new(
                    request.owner,
                    request.creature.clone(),
                    faction,
                    request.duration_seconds,
                ),
                combat_profile,
                CombatStats {
                    current_health: creature.health,
                    max_health: creature.health,
                    resource: 0.0,
                },
                health_pool,
                level,
                CombatAttributes::default(),
                Transform::from_translation(request.spawn_position),
            ));

            match faction {
                SummonFaction::Allies => {
                    entity_commands.insert((
                        AllySummon,
                        AllySummonAi::new(creature.aggro_range),
                    ));
                }
                SummonFaction::Enemies => {
                    entity_commands.insert((
                        Enemy,
                        EnemyAi {
                            target: None,
                            aggro_range: creature.aggro_range,
                            leash_range: creature.leash_range,
                            returning_home: false,
                        },
                        EnemyHomePosition(request.spawn_position),
                    ));
                }
            }
        }
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

    fn spawn_test_app() -> App {
        let database = summon_test_database(
            EffectTarget::Caster,
            true,
        );

        let mut app = App::new();

        app.insert_resource(database)
            .init_resource::<PendingSummons>()
            .add_systems(Update, spawn_pending_summons);

        app
    }

    fn enqueue_spawn_request(
        app: &mut App,
        owner: Entity,
        count: u32,
    ) {
        app.world_mut()
            .resource_mut::<PendingSummons>()
            .enqueue(PendingSummon {
                owner,
                creature: creature_id(),
                spawn_position: Vec3::new(4.0, 2.0, -3.0),
                count,
                duration_seconds: Some(30.0),
            });
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
    fn spawn_system_creates_ally_summons_at_owner_level() {
        let mut app = spawn_test_app();

        let owner = app
            .world_mut()
            .spawn((
                Character,
                CombatLevel::new(60).unwrap(),
                Transform::IDENTITY,
            ))
            .id();

        enqueue_spawn_request(&mut app, owner, 2);

        app.update();

        let world = app.world_mut();
        let mut query = world.query_filtered::<Entity, With<AllySummon>>();
        let entities: Vec<Entity> = query.iter(world).collect();

        assert_eq!(entities.len(), 2);

        for entity in entities {
            let summon = world.get::<SummonedCreature>(entity).unwrap();
            let level = world.get::<CombatLevel>(entity).unwrap();
            let health = world.get::<HealthPool>(entity).unwrap();
            let stats = world.get::<CombatStats>(entity).unwrap();
            let profile = world.get::<SummonCombatProfile>(entity).unwrap();
            let ai = world.get::<AllySummonAi>(entity).unwrap();
            let transform = world.get::<Transform>(entity).unwrap();

            assert_eq!(summon.owner, owner);
            assert_eq!(summon.faction, SummonFaction::Allies);
            assert_eq!(summon.creature, creature_id());
            assert_eq!(summon.remaining_duration(), Some(30.0));
            assert_eq!(level.value(), 60);
            assert_eq!(health.current(), 150.0);
            assert_eq!(health.maximum(), 150.0);
            assert_eq!(stats.current_health, 150.0);
            assert_eq!(stats.max_health, 150.0);
            assert_eq!(profile.attack_damage, 12.0);
            assert_eq!(profile.attack_interval_seconds, 2.0);
            assert_eq!(profile.melee_range, 2.5);
            assert_eq!(profile.aggro_range, 15.0);
            assert_eq!(profile.leash_range, 40.0);
            assert_eq!(profile.movement_speed, 5.0);
            assert_eq!(ai.target, None);
            assert_eq!(ai.aggro_range, 15.0);
            assert_eq!(
                transform.translation,
                Vec3::new(4.0, 2.0, -3.0)
            );
            assert!(world.get::<Enemy>(entity).is_none());
        }

        assert!(world.resource::<PendingSummons>().is_empty());
    }

    #[test]
    fn spawn_system_creates_enemy_summons_with_enemy_components() {
        let mut app = spawn_test_app();

        let owner = app
            .world_mut()
            .spawn((
                Enemy,
                EnemyAi::default(),
                CombatLevel::new(42).unwrap(),
                Transform::IDENTITY,
            ))
            .id();

        enqueue_spawn_request(&mut app, owner, 1);

        app.update();

        let world = app.world_mut();
        let mut query = world.query_filtered::<Entity, With<Enemy>>();
        let entities: Vec<Entity> = query
            .iter(world)
            .filter(|entity| *entity != owner)
            .collect();

        assert_eq!(entities.len(), 1);

        let entity = entities[0];
        let summon = world.get::<SummonedCreature>(entity).unwrap();
        let level = world.get::<CombatLevel>(entity).unwrap();
        let ai = world.get::<EnemyAi>(entity).unwrap();
        let home = world.get::<EnemyHomePosition>(entity).unwrap();
        let profile = world.get::<SummonCombatProfile>(entity).unwrap();

        assert_eq!(profile.attack_damage, 12.0);
        assert_eq!(profile.attack_interval_seconds, 2.0);
        assert_eq!(profile.melee_range, 2.5);
        assert_eq!(profile.aggro_range, 15.0);
        assert_eq!(profile.leash_range, 40.0);
        assert_eq!(profile.movement_speed, 5.0);

        assert_eq!(summon.owner, owner);
        assert_eq!(summon.faction, SummonFaction::Enemies);
        assert_eq!(level.value(), 42);
        assert_eq!(ai.aggro_range, 15.0);
        assert_eq!(ai.leash_range, 40.0);
        assert_eq!(home.0, Vec3::new(4.0, 2.0, -3.0));
        assert!(world.get::<AllySummon>(entity).is_none());
        assert!(world.get::<AllySummonAi>(entity).is_none());
    }

    #[test]
    fn spawned_summons_inherit_the_runtime_faction_of_their_owner() {
        let mut app = spawn_test_app();

        let owner = app
            .world_mut()
            .spawn((
                Enemy,
                EnemyAi::default(),
                SummonedCreature::new(
                    Entity::from_bits(99),
                    creature_id(),
                    SummonFaction::Enemies,
                    None,
                ),
                CombatLevel::new(60).unwrap(),
                Transform::IDENTITY,
            ))
            .id();

        enqueue_spawn_request(&mut app, owner, 1);

        app.update();

        let world = app.world_mut();
        let mut query = world.query_filtered::<Entity, With<SummonedCreature>>();
        let child = query
            .iter(world)
            .find(|entity| *entity != owner)
            .unwrap();

        let child_summon = world.get::<SummonedCreature>(child).unwrap();

        assert_eq!(child_summon.owner, owner);
        assert_eq!(child_summon.faction, SummonFaction::Enemies);
        assert!(world.get::<Enemy>(child).is_some());
        assert!(world.get::<AllySummon>(child).is_none());
        assert!(world.get::<AllySummonAi>(child).is_none());
    }

    #[test]
    fn spawn_system_rejects_owner_without_runtime_faction() {
        let mut app = spawn_test_app();

        let owner = app
            .world_mut()
            .spawn((
                CombatLevel::new(60).unwrap(),
                Transform::IDENTITY,
            ))
            .id();

        enqueue_spawn_request(&mut app, owner, 1);

        app.update();

        let world = app.world_mut();
        let mut query = world.query::<&SummonedCreature>();

        assert_eq!(query.iter(world).count(), 0);
        assert!(world.resource::<PendingSummons>().is_empty());
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
