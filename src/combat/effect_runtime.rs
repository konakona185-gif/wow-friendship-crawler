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
    CombatLevel,
    CombatResources,
    CombatStats,
    EffectExecutionContext,
    EffectResolutionError,
    EffectTargetSelection,
    HealthPool,
    ResolvedEffect,
};

#[derive(Resource, Default)]
pub struct PendingCombatEffects {
    pending: VecDeque<ResolvedEffect>,
}

impl PendingCombatEffects {
    pub fn enqueue(&mut self, effect: ResolvedEffect) {
        self.pending.push_back(effect);
    }

    pub fn enqueue_all(
        &mut self,
        effects: impl IntoIterator<Item = ResolvedEffect>,
    ) {
        self.pending.extend(effects);
    }

    pub fn resolve_and_enqueue_from_database(
        &mut self,
        database: &ContentDatabase,
        effect_id: &ContentId,
        source: Option<Entity>,
        target_selection: EffectTargetSelection,
        source_attributes: &CombatAttributes,
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

        self.enqueue_all(resolved_effects);

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

pub(super) fn execute_pending_effects(
    mut pending: ResMut<PendingCombatEffects>,
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

    for effect in effects {
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
                debug!(
                    effect = ?effect.effect,
                    target = ?effect.target,
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
        ContentMetadata,
        ContentPackManifest,
        EffectDefinition,
        EffectKind,
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

    fn source() -> Entity {
        Entity::from_bits(1)
    }

    fn target() -> Entity {
        Entity::from_bits(2)
    }

    fn target_two() -> Entity {
        Entity::from_bits(3)
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
            .map(|effect| effect.effect.clone())
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

        assert_eq!(queued.effect, effect_id);
        assert_eq!(queued.source, Some(source()));
        assert_eq!(queued.target, target());
        assert_eq!(
            queued.operation,
            super::super::ResolvedEffectOperation::Interrupt
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
            .map(|effect| effect.target)
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
}
