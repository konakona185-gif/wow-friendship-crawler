use std::collections::VecDeque;

use bevy::prelude::*;

use crate::content::ContentDatabase;

use super::{
    execute_resolved_effect,
    CombatAttributes,
    CombatLevel,
    CombatResources,
    CombatStats,
    EffectExecutionContext,
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

    pub fn len(&self) -> usize {
        self.pending.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
}

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
        ContentId,
        EffectTrigger,
    };

    use crate::combat::ResolvedEffectOperation;

    fn effect(id: &str) -> ResolvedEffect {
        ResolvedEffect {
            effect: ContentId::new(
                format!("classic_1_12_1:{id}"),
            )
            .unwrap(),
            trigger: EffectTrigger::OnCast,
            source: None,
            target: Entity::from_bits(1),
            operation: ResolvedEffectOperation::Interrupt,
        }
    }

    #[test]
    fn pending_effects_accept_single_effects() {
        let mut pending = PendingCombatEffects::default();

        pending.enqueue(effect("effect.first"));

        assert_eq!(pending.len(), 1);
        assert!(!pending.is_empty());
    }

    #[test]
    fn pending_effects_accept_multiple_effects() {
        let mut pending = PendingCombatEffects::default();

        pending.enqueue_all(vec![
            effect("effect.first"),
            effect("effect.second"),
            effect("effect.third"),
        ]);

        assert_eq!(pending.len(), 3);
    }

    #[test]
    fn pending_effects_preserve_fifo_order() {
        let mut pending = PendingCombatEffects::default();

        pending.enqueue_all(vec![
            effect("effect.first"),
            effect("effect.second"),
        ]);

        let queued_ids: Vec<ContentId> = pending
            .pending
            .iter()
            .map(|effect| effect.effect.clone())
            .collect();

        assert_eq!(
            queued_ids,
            vec![
                ContentId::new("classic_1_12_1:effect.first").unwrap(),
                ContentId::new("classic_1_12_1:effect.second").unwrap(),
            ]
        );
    }

    #[test]
    fn pending_effects_start_empty() {
        let pending = PendingCombatEffects::default();

        assert!(pending.is_empty());
        assert_eq!(pending.len(), 0);
    }
}
