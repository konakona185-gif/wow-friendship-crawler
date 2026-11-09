use std::collections::VecDeque;
use std::fmt::{Display, Formatter};

use bevy::prelude::*;

use crate::content::{ContentDatabase, ContentId, PeriodicEffectStackScaling};

use crate::combat::aura::{execute_aura_effect, AuraExecutionResult};
use crate::combat::summon::{PendingSummon, PendingSummons};
use crate::combat::{
    execute_resolved_effect, resolve_effect, ActiveAuras, CombatAttributes, CombatEventContext,
    CombatEventOutcome, CombatEventQueue, CombatLevel, CombatResources, CombatStats,
    CustomEffectHandlers, EffectExecutionContext, EffectExecutionResult, EffectResolutionError,
    EffectTargetSelection, EffectTrigger, EnemyThreat, HealthPool, PendingDisplacement,
    ResolvedEffect, ResolvedEffectOperation, ResourceOperation, MAX_REACTIVE_EFFECT_GENERATIONS,
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

    pub fn enqueue_with_generation(&mut self, effect: ResolvedEffect, generation: u8) {
        self.pending.push_back(PendingEffect { effect, generation });
    }

    pub fn enqueue_all(&mut self, effects: impl IntoIterator<Item = ResolvedEffect>) {
        self.enqueue_all_with_generation(effects, 0);
    }

    pub fn enqueue_all_with_generation(
        &mut self,
        effects: impl IntoIterator<Item = ResolvedEffect>,
        generation: u8,
    ) {
        self.pending.extend(
            effects
                .into_iter()
                .map(|effect| PendingEffect { effect, generation }),
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
        periodic_scaling: Option<(u32, PeriodicEffectStackScaling)>,
    ) -> Result<usize, EffectQueueError> {
        let definition = database
            .effect(effect_id)
            .ok_or_else(|| EffectQueueError::MissingEffectDefinition(effect_id.clone()))?;

        let mut resolved_effects =
            resolve_effect(definition, source, target_selection, source_attributes)
                .map_err(EffectQueueError::Resolution)?;

        if let Some((stacks, stack_scaling)) = periodic_scaling {
            for effect in &mut resolved_effects {
                apply_periodic_stack_scaling(effect, stacks, stack_scaling)?;
            }
        }

        let count = resolved_effects.len();

        self.enqueue_all_with_generation(resolved_effects, generation);

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
        ResolvedEffectOperation::Damage(request) => Some(&mut request.base_amount),
        ResolvedEffectOperation::Heal(request) => Some(&mut request.base_amount),
        ResolvedEffectOperation::GainResource(request)
        | ResolvedEffectOperation::SpendResource(request) => Some(&mut request.base_amount),
        ResolvedEffectOperation::ModifyThreat { amount } => Some(amount),
        _ => None,
    };

    if let Some(amount) = amount {
        let scaled_amount = *amount * multiplier;

        if !scaled_amount.is_finite() {
            return Err(EffectQueueError::InvalidPeriodicMagnitude(
                effect.effect.clone(),
            ));
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
                write!(formatter, "Impossibile risolvere l'effetto: {error}")
            }
            Self::ZeroPeriodicStacks => {
                formatter.write_str("Un effetto periodico richiede almeno uno stack")
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
                vec![EffectTrigger::OnDamageDealt, EffectTrigger::OnDamageTaken],
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
                vec![EffectTrigger::OnHealDealt, EffectTrigger::OnHealTaken],
                CombatEventOutcome::Healing(resolution.clone()),
            )
        }
        EffectExecutionResult::Resource(resolution) => {
            if resolution.effective_amount <= 0.0 {
                return;
            }

            let trigger = match resolution.request.operation {
                ResourceOperation::Gain => EffectTrigger::OnResourceGained,
                ResourceOperation::Spend => EffectTrigger::OnResourceSpent,
            };

            (
                vec![trigger],
                CombatEventOutcome::Resource(resolution.clone()),
            )
        }
        EffectExecutionResult::Threat { .. } => {
            return;
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

        if let Err(error) = event_queue.enqueue_with_generation(event, generation) {
            warn!(
                effect = ?effect.effect,
                target = ?effect.target,
                %error,
                "Impossibile registrare l'evento di combattimento."
            );
        }
    }
}

fn enqueue_aura_execution_event(
    event_queue: &mut CombatEventQueue,
    effect: &ResolvedEffect,
    result: &AuraExecutionResult,
    generation: u8,
) {
    let (trigger, outcome) = match result {
        AuraExecutionResult::Applied { aura, total_stacks } => (
            EffectTrigger::OnAuraApplied,
            CombatEventOutcome::AuraApplied {
                aura: aura.clone(),
                total_stacks: *total_stacks,
            },
        ),
        AuraExecutionResult::Removed {
            aura,
            removed_stacks,
        } => {
            if *removed_stacks == 0 {
                return;
            }

            (
                EffectTrigger::OnAuraRemoved,
                CombatEventOutcome::AuraRemoved {
                    aura: aura.clone(),
                    removed_stacks: *removed_stacks,
                },
            )
        }
    };

    let event = CombatEventContext {
        trigger,
        source: effect.source,
        target: Some(effect.target),
        ability: None,
        effect: Some(effect.effect.clone()),
        outcome: Some(outcome),
    };

    if let Err(error) = event_queue.enqueue_with_generation(event, generation) {
        warn!(
            effect = ?effect.effect,
            target = ?effect.target,
            %error,
            "Impossibile registrare l'evento dell'aura."
        );
    }
}

pub(in crate::combat) fn execute_pending_effects(
    mut pending: ResMut<PendingCombatEffects>,
    mut event_queue: ResMut<CombatEventQueue>,
    mut pending_summons: Option<ResMut<PendingSummons>>,
    database: Res<ContentDatabase>,
    custom_handlers: Option<Res<CustomEffectHandlers>>,
    attacker_levels: Query<&CombatLevel>,
    transforms: Query<&Transform>,
    mut target_query: Query<(
        Option<&CombatAttributes>,
        Option<&mut CombatStats>,
        Option<&mut HealthPool>,
        Option<&mut CombatResources>,
        Option<&mut ActiveAuras>,
        Option<&mut EnemyThreat>,
        Option<&mut PendingDisplacement>,
    )>,
) {
    let effects = std::mem::take(&mut pending.pending);

    for queued_effect in effects {
        let effect = queued_effect.effect;
        let generation = queued_effect.generation;

        if let ResolvedEffectOperation::Summon {
            creature,
            count,
            duration_seconds,
        } = &effect.operation
        {
            let Some(owner) = effect.source else {
                warn!(
                    effect = ?effect.effect,
                    target = ?effect.target,
                    "Un effetto summon richiede una sorgente."
                );
                continue;
            };

            if transforms.get(owner).is_err() {
                warn!(
                    effect = ?effect.effect,
                    owner = ?owner,
                    "Impossibile evocare: proprietario ECS non trovato o privo di Transform."
                );
                continue;
            }

            let Ok(spawn_transform) = transforms.get(effect.target) else {
                warn!(
                    effect = ?effect.effect,
                    owner = ?owner,
                    target = ?effect.target,
                    "Impossibile evocare: trasformazione del punto di spawn non trovata."
                );
                continue;
            };

            let spawn_position = spawn_transform.translation;

            if database.creature(creature).is_none() {
                warn!(
                    effect = ?effect.effect,
                    creature = %creature.as_str(),
                    "Impossibile evocare: creatura non registrata nel database dei contenuti."
                );
                continue;
            }

            if *count == 0
                || (*duration_seconds)
                    .is_some_and(|duration| !duration.is_finite() || duration <= 0.0)
                || !spawn_position.is_finite()
            {
                warn!(
                    effect = ?effect.effect,
                    owner = ?owner,
                    target = ?effect.target,
                    count,
                    duration_seconds = ?duration_seconds,
                    spawn_position = ?spawn_position,
                    "Impossibile evocare: parametri o posizione di spawn non validi."
                );
                continue;
            }

            let Some(pending_summons) = pending_summons.as_deref_mut() else {
                warn!(
                    effect = ?effect.effect,
                    "Impossibile accodare l'evocazione: risorsa PendingSummons non inizializzata."
                );
                continue;
            };

            pending_summons.enqueue(PendingSummon {
                owner,
                creature: creature.clone(),
                spawn_position,
                count: *count,
                duration_seconds: *duration_seconds,
            });

            debug!(
                effect = ?effect.effect,
                owner = ?owner,
                target = ?effect.target,
                creature = %creature.as_str(),
                count,
                duration_seconds = ?duration_seconds,
                spawn_position = ?spawn_position,
                generation,
                "Richiesta summon accodata."
            );

            continue;
        }

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
            mut target_threat,
            mut target_displacement,
        )) = target_query.get_mut(effect.target)
        else {
            warn!(
                "Impossibile eseguire l'effetto {:?}: bersaglio ECS non trovato.",
                effect.effect
            );
            continue;
        };

        if let ResolvedEffectOperation::Displacement { distance } = &effect.operation {
            let Some(source) = effect.source else {
                warn!(
                    effect = ?effect.effect,
                    target = ?effect.target,
                    "L'effetto displacement richiede una sorgente."
                );
                continue;
            };

            let (Ok(source_transform), Ok(target_transform)) =
                (transforms.get(source), transforms.get(effect.target))
            else {
                warn!(
                    effect = ?effect.effect,
                    source = ?source,
                    target = ?effect.target,
                    "Impossibile calcolare il displacement: trasformazione sorgente o bersaglio mancante."
                );
                continue;
            };

            let direction =
                (target_transform.translation - source_transform.translation).normalize_or_zero();

            if direction == Vec3::ZERO || !distance.is_finite() || *distance <= 0.0 {
                warn!(
                    effect = ?effect.effect,
                    source = ?source,
                    target = ?effect.target,
                    distance,
                    "Impossibile calcolare un displacement valido."
                );
                continue;
            }

            let Some(displacement) = target_displacement.as_deref_mut() else {
                warn!(
                    effect = ?effect.effect,
                    target = ?effect.target,
                    "Il bersaglio non possiede PendingDisplacement."
                );
                continue;
            };

            let translation = direction * *distance;

            match displacement.accumulate(translation) {
                Ok(()) => {
                    debug!(
                        effect = ?effect.effect,
                        source = ?source,
                        target = ?effect.target,
                        distance,
                        translation = ?translation,
                        generation,
                        "Displacement accodato per il movimento fisico."
                    );
                }
                Err(error) => {
                    warn!(
                        effect = ?effect.effect,
                        target = ?effect.target,
                        %error,
                        "Impossibile accodare il displacement."
                    );
                }
            }

            continue;
        }

        if matches!(
            &effect.operation,
            ResolvedEffectOperation::ApplyAura { .. } | ResolvedEffectOperation::RemoveAura { .. }
        ) {
            match execute_aura_effect(&effect, &database, target_auras.as_deref_mut()) {
                Ok(result) => {
                    enqueue_aura_execution_event(&mut event_queue, &effect, &result, generation);

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

        if let ResolvedEffectOperation::Custom { handler } = &effect.operation {
            let execution_result = {
                let Some(custom_handler) = custom_handlers
                    .as_deref()
                    .and_then(|handlers| handlers.get(handler))
                else {
                    warn!(
                        effect = ?effect.effect,
                        target = ?effect.target,
                        handler = %handler.as_str(),
                        "Handler personalizzato non registrato."
                    );
                    continue;
                };

                let mut context = EffectExecutionContext {
                    database: &database,
                    attacker_level: attacker_level.as_ref(),
                    target_attributes,
                    target_health: target_health.as_deref_mut(),
                    target_resources: target_resources.as_deref_mut(),
                    target_threat: target_threat.as_deref_mut(),
                };

                custom_handler.execute(&effect, &mut context)
            };

            if let (Some(pool), Some(stats)) =
                (target_health.as_deref(), target_stats.as_deref_mut())
            {
                stats.current_health = pool.current();
                stats.max_health = pool.maximum();
            }

            match execution_result {
                Ok(()) => {
                    debug!(
                        effect = ?effect.effect,
                        target = ?effect.target,
                        handler = %handler.as_str(),
                        generation,
                        "Handler personalizzato eseguito."
                    );
                }
                Err(error) => {
                    warn!(
                        effect = ?effect.effect,
                        target = ?effect.target,
                        handler = %handler.as_str(),
                        %error,
                        "Esecuzione dell'handler personalizzato non riuscita."
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
                target_threat: target_threat.as_deref_mut(),
            };

            execute_resolved_effect(&effect, context)
        };

        if let (Some(pool), Some(stats)) = (target_health.as_deref(), target_stats.as_deref_mut()) {
            stats.current_health = pool.current();
            stats.max_health = pool.maximum();
        }

        match execution_result {
            Ok(result) => {
                enqueue_execution_events(&mut event_queue, &effect, &result, generation);

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
mod tests;
