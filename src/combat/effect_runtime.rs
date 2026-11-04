
use std::collections::VecDeque;
use std::fmt::{Display, Formatter};

use bevy::prelude::*;

use crate::content::{
    ContentDatabase,
    ContentId,
    PeriodicEffectStackScaling,
};

use super::{
    aura_execution::execute_aura_effect,
    execute_resolved_effect,
    resolve_effect,
    ActiveAuras,
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
    ResolvedEffectOperation,
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
        self.resolve_and_enqueue_internal(
            database,
            effect_id,
            source,
            target_selection,
            source_attributes,
            generation,
            None,
        )
    }

    pub fn resolve_and_enqueue_periodic_from_database_with_generation(
        &mut self,
        database: &ContentDatabase,
        effect_id: &ContentId,
        source: Option<Entity>,
        target_selection: EffectTargetSelection,
        source_attributes: &CombatAttributes,
        stacks: u32,
        stack_scaling: PeriodicEffectStackScaling,
        generation: u8,
    ) -> Result<usize, EffectQueueError> {
        self.resolve_and_enqueue_internal(
            database,
            effect_id,
            source,
            target_selection,
            source_attributes,
            generation,
            Some((stacks, stack_scaling)),
        )
    }

    fn resolve_and_enqueue_internal(
        &mut self,
        database: &ContentDatabase,
        effect_id: &ContentId,
        source: Option<Entity>,
        target_selection: EffectTargetSelection,
        source_attributes: &CombatAttributes,
        generation: u8,
        periodic_scaling: Option<(
            u32,
            PeriodicEffectStackScaling,
        )>,
    ) -> Result<usize, EffectQueueError> {
        let definition = database
            .effect(effect_id)
            .ok_or_else(|| {
                EffectQueueError::MissingEffectDefinition(
                    effect_id.clone(),
                )
            })?;

        let mut resolved_effects = resolve_effect(
            definition,
            source,
            target_selection,
            source_attributes,
        )
        .map_err(EffectQueueError::Resolution)?;

        if let Some((stacks, stack_scaling)) = periodic_scaling {
            for effect in &mut resolved_effects {
                apply_periodic_stack_scaling(
                    effect,
                    stacks,
                    stack_scaling,
                )?;
            }
        }

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

fn apply_periodic_stack_scaling(
    effect: &mut ResolvedEffect,
    stacks: u32,
    stack_scaling: PeriodicEffectStackScaling,
) -> Result<(), EffectQueueError> {
    if stacks == 0 {
        return Err(EffectQueueError::ZeroPeriodicStacks);
    }

    let multiplier = match stack_scaling {
        PeriodicEffectStackScaling::None => 1.0,
        PeriodicEffectStackScaling::Linear => stacks as f32,
    };

    let amount = match &mut effect.operation {
        ResolvedEffectOperation::Damage(request) => {
            Some(&mut request.base_amount)
        }
        ResolvedEffectOperation::Heal(request) => {
            Some(&mut request.base_amount)
        }
        ResolvedEffectOperation::GainResource(request)
        | ResolvedEffectOperation::SpendResource(request) => {
            Some(&mut request.base_amount)
        }
        ResolvedEffectOperation::ModifyThreat { amount } => {
            Some(amount)
        }
        _ => None,
    };

    if let Some(amount) = amount {
        let scaled_amount = *amount * multiplier;

        if !scaled_amount.is_finite() {
            return Err(
                EffectQueueError::InvalidPeriodicMagnitude(
                    effect.effect.clone(),
                )
            );
        }

        *amount = scaled_amount;
    }

    Ok(())
}

#[derive(Clone, Debug, PartialEq)]
pub enum EffectQueueError {
    MissingEffectDefinition(ContentId),
    Resolution(EffectResolutionError),
    ZeroPeriodicStacks,
    InvalidPeriodicMagnitude(ContentId),
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
            Self::ZeroPeriodicStacks => {
                formatter.write_str(
                    "Un effetto periodico richiede almeno uno stack",
                )
            }
            Self::InvalidPeriodicMagnitude(id) => {
                write!(
                    formatter,
                    "La scalatura periodica ha prodotto un valore non valido per l'effetto: {}",
                    id.as_str()
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
        Option<&mut ActiveAuras>,
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
            mut target_auras,
        )) = target_query.get_mut(effect.target)
        else {
            warn!(
                "Impossibile eseguire l'effetto {:?}: bersaglio ECS non trovato.",
                effect.effect
            );
            continue;
        };

        if matches!(
            &effect.operation,
            ResolvedEffectOperation::ApplyAura { .. }
                | ResolvedEffectOperation::RemoveAura { .. }
        ) {
            match execute_aura_effect(
                &effect,
                &database,
                target_auras.as_deref_mut(),
            ) {
                Ok(result) => {
                    debug!(
                        effect = ?effect.effect,
                        target = ?effect.target,
                        generation,
                        result = ?result,
                        "Effetto aura di combattimento eseguito."
                    );
                }
                Err(error) => {
                    warn!(
                        effect = ?effect.effect,
                        target = ?effect.target,
                        %error,
                        "Esecuzione dell'effetto aura non riuscita."
                    );
                }
            }

            continue;
        }

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
#[path = "effect_runtime/aura_tests.rs"]
mod aura_tests;

#[cfg(test)]
mod tests;
