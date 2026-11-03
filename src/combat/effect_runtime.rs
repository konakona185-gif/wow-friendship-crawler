use std::collections::VecDeque;
use std::fmt::{Display, Formatter};

use bevy::prelude::*;

use crate::content::{
    ContentDatabase,
    ContentId,
};

use super::{
    execute_resolved_effect,
    resolve_effect,
    CombatAttributes,
    CombatEventContext,
    CombatEventOutcome,
    CombatEventQueue,
    CombatLevel,
    CombatResources,
    CombatStats,
    EffectExecutionContext,
    EffectExecutionResult,
    EffectResolutionError,
    EffectTargetSelection,
    EffectTrigger,
    HealthPool,
    ResolvedEffect,
    ResourceOperation,
    MAX_REACTIVE_EFFECT_GENERATIONS,
};

#[derive(Clone, Debug, PartialEq)]
struct PendingEffect {
    effect: ResolvedEffect,
    generation: u8,
}

#[derive(Resource, Default)]
pub struct PendingCombatEffects {
    pending: VecDeque<PendingEffect>,
}

impl PendingCombatEffects {
    pub fn enqueue(&mut self, effect: ResolvedEffect) {
        self.enqueue_with_generation(effect, 0);
    }

    pub fn enqueue_with_generation(
        &mut self,
        effect: ResolvedEffect,
        generation: u8,
    ) {
        self.pending.push_back(PendingEffect {
            effect,
            generation,
        });
    }

    pub fn enqueue_all(
        &mut self,
        effects: impl IntoIterator<Item = ResolvedEffect>,
    ) {
        self.enqueue_all_with_generation(effects, 0);
    }

    pub fn enqueue_all_with_generation(
        &mut self,
        effects: impl IntoIterator<Item = ResolvedEffect>,
        generation: u8,
    ) {
        self.pending.extend(
            effects.into_iter().map(|effect| PendingEffect {
                effect,
                generation,
            }),
        );
    }

    pub fn resolve_and_enqueue_from_database(
        &mut self,
        database: &ContentDatabase,
        effect_id: &ContentId,
        source: Option<Entity>,
        target_selection: EffectTargetSelection,
        source_attributes: &CombatAttributes,
    ) -> Result<usize, EffectQueueError> {
        self.resolve_and_enqueue_from_database_with_generation(
            database,
            effect_id,
            source,
            target_selection,
            source_attributes,
            0,
        )
    }

    pub fn resolve_and_enqueue_from_database_with_generation(
        &mut self,
        database: &ContentDatabase,
        effect_id: &ContentId,
        source: Option<Entity>,
        target_selection: EffectTargetSelection,
        source_attributes: &CombatAttributes,
        generation: u8,
    ) -> Result<usize, EffectQueueError> {
        let definition = database
            .effect(effect_id)
            .ok_or_else(|| {
                EffectQueueError::MissingEffectDefinition(
                    effect_id.clone(),
                )
            })?;

        let resolved_effects = resolve_effect(
            definition,
            source,
            target_selection,
            source_attributes,
        )
        .map_err(EffectQueueError::Resolution)?;

        let count = resolved_effects.len();

        self.enqueue_all_with_generation(
            resolved_effects,
            generation,
        );

        Ok(count)
    }

    pub fn len(&self) -> usize {
        self.pending.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum EffectQueueError {
    MissingEffectDefinition(ContentId),
    Resolution(EffectResolutionError),
}

impl Display for EffectQueueError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingEffectDefinition(id) => {
                write!(
                    formatter,
                    "Definizione dell'effetto non registrata: {}",
                    id.as_str()
                )
            }
            Self::Resolution(error) => {
                write!(
                    formatter,
                    "Impossibile risolvere l'effetto: {error}"
                )
            }
        }
    }
}

impl std::error::Error for EffectQueueError {}

fn enqueue_execution_events(
    event_queue: &mut CombatEventQueue,
    effect: &ResolvedEffect,
    result: &EffectExecutionResult,
    generation: u8,
) {
    let (triggers, outcome) = match result {
        EffectExecutionResult::Damage {
            resolution,
            applied_amount,
        } => {
            if *applied_amount <= 0.0 {
                return;
            }

            (
                vec![
                    EffectTrigger::OnDamageDealt,
                    EffectTrigger::OnDamageTaken,
                ],
                CombatEventOutcome::Damage(resolution.clone()),
            )
        }
        EffectExecutionResult::Healing {
            resolution,
            applied_amount,
        } => {
            if *applied_amount <= 0.0 {
                return;
            }

            (
                vec![
                    EffectTrigger::OnHealDealt,
                    EffectTrigger::OnHealTaken,
                ],
                CombatEventOutcome::Healing(resolution.clone()),
            )
        }
        EffectExecutionResult::Resource(resolution) => {
            if resolution.effective_amount <= 0.0 {
                return;
            }

            let trigger = match resolution.request.operation {
                ResourceOperation::Gain => {
                    EffectTrigger::OnResourceGained
                }
                ResourceOperation::Spend => {
                    EffectTrigger::OnResourceSpent
                }
            };

            (
                vec![trigger],
                CombatEventOutcome::Resource(resolution.clone()),
            )
        }
    };

    for trigger in triggers {
        let event = CombatEventContext {
            trigger,
            source: effect.source,
            target: Some(effect.target),
            ability: None,
            effect: Some(effect.effect.clone()),
            outcome: Some(outcome.clone()),
        };

        if let Err(error) = event_queue.enqueue_with_generation(
            event,
            generation,
        ) {
            warn!(
                effect = ?effect.effect,
                target = ?effect.target,
                %error,
                "Impossibile registrare l'evento di combattimento."
            );
        }
    }
}

pub(super) fn execute_pending_effects(
    mut pending: ResMut<PendingCombatEffects>,
    mut event_queue: ResMut<CombatEventQueue>,
    database: Res<ContentDatabase>,
    attacker_levels: Query<&CombatLevel>,
    mut target_query: Query<(
        Option<&CombatAttributes>,
        Option<&mut CombatStats>,
        Option<&mut HealthPool>,
        Option<&mut CombatResources>,
    )>,
) {
    let effects = std::mem::take(&mut pending.pending);

    for queued_effect in effects {
        let effect = queued_effect.effect;
        let generation = queued_effect.generation;

        let attacker_level = effect
            .source
            .and_then(|source| attacker_levels.get(source).ok())
            .copied();

        let Ok((
            target_attributes,
            mut target_stats,
            mut target_health,
            mut target_resources,
        )) = target_query.get_mut(effect.target)
        else {
            warn!(
                "Impossibile eseguire l'effetto {:?}: bersaglio ECS non trovato.",
                effect.effect
            );
            continue;
        };

        let execution_result = {
            let context = EffectExecutionContext {
                database: &database,
                attacker_level: attacker_level.as_ref(),
                target_attributes,
                target_health: target_health.as_deref_mut(),
                target_resources: target_resources.as_deref_mut(),
            };

            execute_resolved_effect(&effect, context)
        };

        if let (Some(pool), Some(stats)) = (
            target_health.as_deref(),
            target_stats.as_deref_mut(),
        ) {
            stats.current_health = pool.current();
            stats.max_health = pool.maximum();
        }

        match execution_result {
            Ok(result) => {
                enqueue_execution_events(
                    &mut event_queue,
                    &effect,
                    &result,
                    generation,
                );

                debug!(
                    effect = ?effect.effect,
                    target = ?effect.target,
                    generation,
                    result = ?result,
                    "Effetto di combattimento eseguito."
                );
            }
            Err(error) => {
                warn!(
                    effect = ?effect.effect,
                    target = ?effect.target,
                    %error,
                    "Esecuzione dell'effetto di combattimento non riuscita."
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::content::{
        CombatAttributeDefinition,
        CombatAttributeValueKind,
        ContentMetadata,
        ContentPackManifest,
        DamageSchoolDefinition,
        DamageSchoolMitigation,
        EffectDefinition,
        EffectKind,
        EffectMagnitude,
        EffectTarget,
        EffectTrigger,
    };

    fn content_id(value: &str) -> ContentId {
        ContentId::new(
            format!("classic_1_12_1:{value}"),
        )
        .unwrap()
    }

    fn test_database(
        target: EffectTarget,
    ) -> (ContentDatabase, ContentId) {
        let mut database = ContentDatabase::new();

        let pack_id = ContentId::new(
            "pack:classic_1_12_1",
        )
        .unwrap();

        database
            .register_pack(ContentPackManifest {
                id: pack_id.clone(),
                name: "Classic 1.12.1".to_string(),
                version: "1.0.0".to_string(),
                description: String::new(),
                dependencies: Vec::new(),
            })
            .unwrap();

        let effect_id = content_id("effect.test");

        database
            .register_effect(EffectDefinition {
                metadata: ContentMetadata {
                    id: effect_id.clone(),
                    pack_id,
                    name: "Test Effect".to_string(),
                    description: String::new(),
                    icon: None,
                    tags: Vec::new(),
                },
                trigger: EffectTrigger::OnCast,
                target,
                kind: EffectKind::Interrupt,
            })
            .unwrap();

        (database, effect_id)
    }

    fn damage_test_database() -> (ContentDatabase, ContentId) {
        let mut database = ContentDatabase::new();

        let pack_id = ContentId::new(
            "pack:classic_1_12_1",
        )
        .unwrap();

        database
            .register_pack(ContentPackManifest {
                id: pack_id.clone(),
                name: "Classic 1.12.1".to_string(),
                version: "1.0.0".to_string(),
                description: String::new(),
                dependencies: Vec::new(),
            })
            .unwrap();

        let armor_id = content_id("stat.armor");

        database
            .register_combat_attribute(CombatAttributeDefinition {
                metadata: ContentMetadata {
                    id: armor_id,
                    pack_id: pack_id.clone(),
                    name: "Armor".to_string(),
                    description: String::new(),
                    icon: None,
                    tags: Vec::new(),
                },
                value_kind: CombatAttributeValueKind::Rating,
            })
            .unwrap();

        let school_id = content_id("school.physical");

        database
            .register_damage_school(DamageSchoolDefinition {
                metadata: ContentMetadata {
                    id: school_id.clone(),
                    pack_id: pack_id.clone(),
                    name: "Physical".to_string(),
                    description: String::new(),
                    icon: None,
                    tags: Vec::new(),
                },
                mitigation: DamageSchoolMitigation::Armor,
            })
            .unwrap();

        let effect_id = content_id("effect.damage_test");

        database
            .register_effect(EffectDefinition {
                metadata: ContentMetadata {
                    id: effect_id.clone(),
                    pack_id,
                    name: "Damage Test".to_string(),
                    description: String::new(),
                    icon: None,
                    tags: Vec::new(),
                },
                trigger: EffectTrigger::OnCast,
                target: EffectTarget::SelectedTarget,
                kind: EffectKind::Damage {
                    amount: EffectMagnitude {
                        base: 40.0,
                        scaling: Vec::new(),
                    },
                    school: school_id,
                },
            })
            .unwrap();

        database.validate().unwrap();

        (database, effect_id)
    }

    fn source() -> Entity {
        Entity::from_bits(1)
    }

    fn target() -> Entity {
        Entity::from_bits(2)
    }

    fn target_two() -> Entity {
        Entity::from_bits(3)
    }

    fn event_test_effect() -> ResolvedEffect {
        ResolvedEffect {
            effect: content_id("effect.event_test"),
            trigger: EffectTrigger::OnCast,
            source: Some(source()),
            target: target(),
            operation: super::super::ResolvedEffectOperation::Interrupt,
        }
    }

    #[test]
    fn pending_effects_accept_single_effects() {
        let mut pending = PendingCombatEffects::default();

        pending.enqueue(ResolvedEffect {
            effect: content_id("effect.first"),
            trigger: EffectTrigger::OnCast,
            source: None,
            target: target(),
            operation: super::super::ResolvedEffectOperation::Interrupt,
        });

        assert_eq!(pending.len(), 1);
        assert!(!pending.is_empty());

        assert_eq!(
            pending.pending.front().unwrap().generation,
            0
        );
    }

    #[test]
    fn pending_effects_accept_multiple_effects() {
        let mut pending = PendingCombatEffects::default();

        pending.enqueue_all(vec![
            ResolvedEffect {
                effect: content_id("effect.first"),
                trigger: EffectTrigger::OnCast,
                source: None,
                target: target(),
                operation: super::super::ResolvedEffectOperation::Interrupt,
            },
            ResolvedEffect {
                effect: content_id("effect.second"),
                trigger: EffectTrigger::OnCast,
                source: None,
                target: target_two(),
                operation: super::super::ResolvedEffectOperation::Interrupt,
            },
        ]);

        assert_eq!(pending.len(), 2);

        assert!(
            pending
                .pending
                .iter()
                .all(|effect| effect.generation == 0)
        );
    }

    #[test]
    fn pending_effects_preserve_fifo_order() {
        let mut pending = PendingCombatEffects::default();

        pending.enqueue_all(vec![
            ResolvedEffect {
                effect: content_id("effect.first"),
                trigger: EffectTrigger::OnCast,
                source: None,
                target: target(),
                operation: super::super::ResolvedEffectOperation::Interrupt,
            },
            ResolvedEffect {
                effect: content_id("effect.second"),
                trigger: EffectTrigger::OnCast,
                source: None,
                target: target_two(),
                operation: super::super::ResolvedEffectOperation::Interrupt,
            },
        ]);

        let queued_ids: Vec<ContentId> = pending
            .pending
            .iter()
            .map(|queued| queued.effect.effect.clone())
            .collect();

        assert_eq!(
            queued_ids,
            vec![
                content_id("effect.first"),
                content_id("effect.second"),
            ]
        );
    }

    #[test]
    fn pending_effects_preserve_explicit_generation() {
        let mut pending = PendingCombatEffects::default();

        pending.enqueue_with_generation(
            ResolvedEffect {
                effect: content_id("effect.first"),
                trigger: EffectTrigger::OnCast,
                source: None,
                target: target(),
                operation: super::super::ResolvedEffectOperation::Interrupt,
            },
            5,
        );

        assert_eq!(
            pending.pending.front().unwrap().generation,
            5
        );
    }

    #[test]
    fn pending_effects_apply_generation_to_multiple_effects() {
        let mut pending = PendingCombatEffects::default();

        pending.enqueue_all_with_generation(
            vec![
                ResolvedEffect {
                    effect: content_id("effect.first"),
                    trigger: EffectTrigger::OnCast,
                    source: None,
                    target: target(),
                    operation: super::super::ResolvedEffectOperation::Interrupt,
                },
                ResolvedEffect {
                    effect: content_id("effect.second"),
                    trigger: EffectTrigger::OnCast,
                    source: None,
                    target: target_two(),
                    operation: super::super::ResolvedEffectOperation::Interrupt,
                },
            ],
            3,
        );

        assert_eq!(pending.len(), 2);
        assert!(
            pending
                .pending
                .iter()
                .all(|effect| effect.generation == 3)
        );
    }

    #[test]
    fn pending_effects_start_empty() {
        let pending = PendingCombatEffects::default();

        assert!(pending.is_empty());
        assert_eq!(pending.len(), 0);
    }

    #[test]
    fn database_effects_are_resolved_and_enqueued() {
        let (database, effect_id) = test_database(
            EffectTarget::SelectedTarget,
        );

        let mut pending = PendingCombatEffects::default();

        let count = pending
            .resolve_and_enqueue_from_database(
                &database,
                &effect_id,
                Some(source()),
                EffectTargetSelection::Selected(target()),
                &CombatAttributes::default(),
            )
            .unwrap();

        assert_eq!(count, 1);
        assert_eq!(pending.len(), 1);

        let queued = pending.pending.front().unwrap();

        assert_eq!(queued.effect.effect, effect_id);
        assert_eq!(queued.effect.source, Some(source()));
        assert_eq!(queued.effect.target, target());
        assert_eq!(
            queued.effect.operation,
            super::super::ResolvedEffectOperation::Interrupt
        );
        assert_eq!(queued.generation, 0);
    }

    #[test]
    fn database_effects_preserve_requested_generation() {
        let (database, effect_id) = test_database(
            EffectTarget::SelectedTarget,
        );

        let mut pending = PendingCombatEffects::default();

        let count = pending
            .resolve_and_enqueue_from_database_with_generation(
                &database,
                &effect_id,
                Some(source()),
                EffectTargetSelection::Selected(target()),
                &CombatAttributes::default(),
                4,
            )
            .unwrap();

        assert_eq!(count, 1);
        assert_eq!(
            pending.pending.front().unwrap().generation,
            4
        );
    }

    #[test]
    fn area_effects_are_enqueued_in_target_order() {
        let (database, effect_id) = test_database(
            EffectTarget::EnemiesInRadius {
                radius: 10.0,
            },
        );

        let mut pending = PendingCombatEffects::default();

        let count = pending
            .resolve_and_enqueue_from_database(
                &database,
                &effect_id,
                Some(source()),
                EffectTargetSelection::Area(vec![
                    target(),
                    target_two(),
                ]),
                &CombatAttributes::default(),
            )
            .unwrap();

        assert_eq!(count, 2);
        assert_eq!(pending.len(), 2);

        let targets: Vec<Entity> = pending
            .pending
            .iter()
            .map(|queued| queued.effect.target)
            .collect();

        assert_eq!(
            targets,
            vec![target(), target_two()]
        );
    }

    #[test]
    fn missing_effect_definition_does_not_change_queue() {
        let (database, _) = test_database(
            EffectTarget::SelectedTarget,
        );

        let mut pending = PendingCombatEffects::default();

        pending.enqueue(ResolvedEffect {
            effect: content_id("effect.existing"),
            trigger: EffectTrigger::OnCast,
            source: None,
            target: target(),
            operation: super::super::ResolvedEffectOperation::Interrupt,
        });

        let result = pending.resolve_and_enqueue_from_database(
            &database,
            &content_id("effect.missing"),
            Some(source()),
            EffectTargetSelection::Selected(target_two()),
            &CombatAttributes::default(),
        );

        assert_eq!(
            result,
            Err(EffectQueueError::MissingEffectDefinition(
                content_id("effect.missing")
            ))
        );

        assert_eq!(pending.len(), 1);
    }

    #[test]
    fn failed_resolution_does_not_change_queue() {
        let (database, effect_id) = test_database(
            EffectTarget::SelectedTarget,
        );

        let mut pending = PendingCombatEffects::default();

        let result = pending.resolve_and_enqueue_from_database(
            &database,
            &effect_id,
            Some(source()),
            EffectTargetSelection::Caster,
            &CombatAttributes::default(),
        );

        assert_eq!(
            result,
            Err(EffectQueueError::Resolution(
                EffectResolutionError::TargetSelectionMismatch
            ))
        );

        assert!(pending.is_empty());
    }

    #[test]
    fn runtime_executes_damage_and_synchronizes_health_components() {
        let (database, effect_id) = damage_test_database();

        let mut app = App::new();

        let source_entity = app
            .world_mut()
            .spawn(CombatLevel::new(60).unwrap())
            .id();

        let target_entity = app
            .world_mut()
            .spawn((
                CombatAttributes::default(),
                CombatStats {
                    current_health: 100.0,
                    max_health: 100.0,
                    resource: 0.0,
                },
                HealthPool::new(100.0, 100.0).unwrap(),
            ))
            .id();

        let mut pending = PendingCombatEffects::default();

        let count = pending
            .resolve_and_enqueue_from_database(
                &database,
                &effect_id,
                Some(source_entity),
                EffectTargetSelection::Selected(target_entity),
                &CombatAttributes::default(),
            )
            .unwrap();

        assert_eq!(count, 1);

        app.insert_resource(database)
            .insert_resource(pending)
            .init_resource::<CombatEventQueue>()
            .add_systems(Update, execute_pending_effects);

        app.update();

        {
            let world = app.world();

            let health = world
                .get::<HealthPool>(target_entity)
                .unwrap();

            let stats = world
                .get::<CombatStats>(target_entity)
                .unwrap();

            assert_eq!(health.current(), 60.0);
            assert_eq!(health.maximum(), 100.0);
            assert_eq!(stats.current_health, 60.0);
            assert_eq!(stats.max_health, 100.0);

            assert!(
                world
                    .resource::<PendingCombatEffects>()
                    .is_empty()
            );
        }

        let events = app
            .world_mut()
            .resource_mut::<CombatEventQueue>()
            .drain_with_generation();

        assert_eq!(events.len(), 2);
        assert_eq!(
            events[0].0.trigger,
            EffectTrigger::OnDamageDealt
        );
        assert_eq!(
            events[1].0.trigger,
            EffectTrigger::OnDamageTaken
        );

        for (event, generation) in &events {
            assert_eq!(event.source, Some(source_entity));
            assert_eq!(event.target, Some(target_entity));
            assert_eq!(event.effect, Some(effect_id.clone()));
            assert_eq!(*generation, 0);
            assert!(matches!(
                &event.outcome,
                Some(CombatEventOutcome::Damage(_))
            ));
        }
    }

    #[test]
    fn runtime_preserves_generation_when_emitting_events() {
        let request = super::super::DamageRequest {
            source: Some(source()),
            target: target(),
            base_amount: 100.0,
            school: content_id("school.physical"),
        };

        let resolution = super::super::DamageResolution::new(
            request,
            super::super::DamageMitigationResult::new(
                100.0,
                25.0,
            )
            .unwrap(),
        )
        .unwrap();

        let result = EffectExecutionResult::Damage {
            resolution,
            applied_amount: 75.0,
        };

        let effect = event_test_effect();
        let mut event_queue = CombatEventQueue::default();

        enqueue_execution_events(
            &mut event_queue,
            &effect,
            &result,
            6,
        );

        let events = event_queue.drain_with_generation();

        assert_eq!(events.len(), 2);
        assert!(
            events
                .iter()
                .all(|(_, generation)| *generation == 6)
        );
    }

    #[test]
    fn runtime_does_not_emit_damage_events_when_no_damage_is_applied() {
        let (database, effect_id) = damage_test_database();

        let mut app = App::new();

        let source_entity = app
            .world_mut()
            .spawn(CombatLevel::new(60).unwrap())
            .id();

        let target_entity = app
            .world_mut()
            .spawn((
                CombatAttributes::default(),
                CombatStats {
                    current_health: 0.0,
                    max_health: 100.0,
                    resource: 0.0,
                },
                HealthPool::new(0.0, 100.0).unwrap(),
            ))
            .id();

        let mut pending = PendingCombatEffects::default();

        pending
            .resolve_and_enqueue_from_database(
                &database,
                &effect_id,
                Some(source_entity),
                EffectTargetSelection::Selected(target_entity),
                &CombatAttributes::default(),
            )
            .unwrap();

        app.insert_resource(database)
            .insert_resource(pending)
            .init_resource::<CombatEventQueue>()
            .add_systems(Update, execute_pending_effects);

        app.update();

        let events = app
            .world_mut()
            .resource_mut::<CombatEventQueue>()
            .drain();

        assert!(events.is_empty());

        let world = app.world();
        let health = world
            .get::<HealthPool>(target_entity)
            .unwrap();

        assert_eq!(health.current(), 0.0);
    }

    #[test]
    fn runtime_emits_healing_dealt_and_taken_events() {
        let request = super::super::HealingRequest {
            source: Some(source()),
            target: target(),
            base_amount: 25.0,
        };

        let resolution = super::super::HealingResolution::new(
            request,
            25.0,
        )
        .unwrap();

        let result = EffectExecutionResult::Healing {
            resolution,
            applied_amount: 25.0,
        };

        let effect = event_test_effect();
        let mut event_queue = CombatEventQueue::default();

        enqueue_execution_events(
            &mut event_queue,
            &effect,
            &result,
            0,
        );

        let events = event_queue.drain();

        assert_eq!(events.len(), 2);
        assert_eq!(
            events[0].trigger,
            EffectTrigger::OnHealDealt
        );
        assert_eq!(
            events[1].trigger,
            EffectTrigger::OnHealTaken
        );

        for event in &events {
            assert_eq!(event.source, Some(source()));
            assert_eq!(event.target, Some(target()));
            assert!(matches!(
                &event.outcome,
                Some(CombatEventOutcome::Healing(_))
            ));
        }
    }

    #[test]
    fn runtime_uses_the_correct_resource_event_trigger() {
        for (operation, expected_trigger) in [
            (
                ResourceOperation::Gain,
                EffectTrigger::OnResourceGained,
            ),
            (
                ResourceOperation::Spend,
                EffectTrigger::OnResourceSpent,
            ),
        ] {
            let request = super::super::ResourceRequest {
                source: Some(source()),
                target: target(),
                resource: content_id("resource.mana"),
                operation,
                base_amount: 20.0,
            };

            let resolution = super::super::ResourceResolution::new(
                request,
                20.0,
            )
            .unwrap();

            let result = EffectExecutionResult::Resource(resolution);

            let effect = event_test_effect();
            let mut event_queue = CombatEventQueue::default();

            enqueue_execution_events(
                &mut event_queue,
                &effect,
                &result,
                0,
            );

            let events = event_queue.drain();

            assert_eq!(events.len(), 1);
            assert_eq!(events[0].trigger, expected_trigger);
            assert_eq!(events[0].source, Some(source()));
            assert_eq!(events[0].target, Some(target()));
            assert!(matches!(
                &events[0].outcome,
                Some(CombatEventOutcome::Resource(_))
            ));
        }
    }

    #[test]
    fn runtime_does_not_emit_healing_or_resource_events_for_zero_amounts() {
        let healing_request = super::super::HealingRequest {
            source: Some(source()),
            target: target(),
            base_amount: 25.0,
        };

        let healing_resolution = super::super::HealingResolution::new(
            healing_request,
            0.0,
        )
        .unwrap();

        let healing_result = EffectExecutionResult::Healing {
            resolution: healing_resolution,
            applied_amount: 0.0,
        };

        let resource_request = super::super::ResourceRequest {
            source: Some(source()),
            target: target(),
            resource: content_id("resource.mana"),
            operation: ResourceOperation::Gain,
            base_amount: 20.0,
        };

        let resource_resolution = super::super::ResourceResolution::new(
            resource_request,
            0.0,
        )
        .unwrap();

        let resource_result = EffectExecutionResult::Resource(
            resource_resolution,
        );

        let effect = event_test_effect();
        let mut event_queue = CombatEventQueue::default();

        enqueue_execution_events(
            &mut event_queue,
            &effect,
            &healing_result,
            0,
        );

        enqueue_execution_events(
            &mut event_queue,
            &effect,
            &resource_result,
            0,
        );

        assert!(event_queue.is_empty());
    }

    #[test]
    fn generation_limit_is_available_to_runtime() {
        assert_eq!(MAX_REACTIVE_EFFECT_GENERATIONS, 16);
    }
}
