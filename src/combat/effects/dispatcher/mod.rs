use bevy::prelude::*;

use crate::content::{
    ContentDatabase,
    EffectTarget,
    EffectTrigger,
};

use super::acquire_area_targets_from_world;
use crate::combat::{
    AllySummon,
    Character,
    CombatAttributes,
    CombatEffectSubscriptions,
    CombatEventContext,
    CombatEventOutcome,
    CombatEventQueue,
    CombatStats,
    EffectTargetSelection,
    Enemy,
    HealthPool,
    PendingCombatEffects,
    MAX_REACTIVE_EFFECT_GENERATIONS,
};

fn event_participants(
    event: &CombatEventContext,
) -> Option<(Entity, Option<Entity>)> {
    match event.trigger {
        EffectTrigger::OnDamageDealt
        | EffectTrigger::OnHealDealt
        | EffectTrigger::OnPeriodicTick => {
            Some((event.source?, event.target))
        }
        EffectTrigger::OnDamageTaken
        | EffectTrigger::OnHealTaken
        | EffectTrigger::OnResourceGained
        | EffectTrigger::OnResourceSpent
        | EffectTrigger::OnExpire
        | EffectTrigger::OnAuraApplied
        | EffectTrigger::OnAuraRemoved => {
            Some((event.target?, event.source))
        }
        _ => None,
    }
}

fn event_matches_effect(
    event: &CombatEventContext,
    effect_id: &crate::content::ContentId,
) -> bool {
    if event.trigger != EffectTrigger::OnPeriodicTick {
        return true;
    }

    event
        .effect
        .as_ref()
        .is_some_and(|triggered_effect| triggered_effect == effect_id)
}

pub(super) fn dispatch_combat_events(
    mut event_queue: ResMut<CombatEventQueue>,
    database: Res<ContentDatabase>,
    mut pending: ResMut<PendingCombatEffects>,
    subscribers: Query<(
        &CombatEffectSubscriptions,
        Option<&CombatAttributes>,
    )>,
    combatants: Query<(
        Entity,
        &Transform,
        Option<&CombatStats>,
        Option<&HealthPool>,
        Option<&Character>,
        Option<&Enemy>,
        Option<&AllySummon>,
    ), Or<(With<Character>, With<Enemy>, With<AllySummon>)>>,
) {
    let events = event_queue.drain_with_generation();

    for (event, generation) in events {
        if generation >= MAX_REACTIVE_EFFECT_GENERATIONS {
            continue;
        }

        let Some((subscriber, other_participant)) =
            event_participants(&event)
        else {
            continue;
        };

        let Ok((subscriptions, attributes)) = subscribers.get(subscriber)
        else {
            continue;
        };

        let next_generation = generation + 1;
        let source_attributes =
            attributes.cloned().unwrap_or_default();

        for effect_id in subscriptions.iter() {
            let Some(definition) = database.effect(effect_id) else {
                warn!(
                    "Effetto reattivo non registrato: {}",
                    effect_id.as_str()
                );
                continue;
            };

            if definition.trigger != event.trigger
                || !event_matches_effect(&event, effect_id)
            {
                continue;
            }

            let target_selection = match &definition.target {
                EffectTarget::Caster => EffectTargetSelection::Caster,
                EffectTarget::SelectedTarget => {
                    let Some(target) = other_participant else {
                        continue;
                    };

                    EffectTargetSelection::Selected(target)
                }
                EffectTarget::AlliesInRadius { .. }
                | EffectTarget::EnemiesInRadius { .. }
                | EffectTarget::UnitsInRadius { .. } => {
                    match acquire_area_targets_from_world(
                        &definition.target,
                        subscriber,
                        &combatants,
                    ) {
                        Ok(selection) => selection,
                        Err(error) => {
                            warn!(
                                effect = ?effect_id,
                                error = ?error,
                                "Acquisizione dei bersagli ad area non riuscita."
                            );

                            continue;
                        }
                    }
                }
            };

            let enqueue_result =
                if event.trigger == EffectTrigger::OnPeriodicTick {
                    let Some(
                        CombatEventOutcome::PeriodicTick {
                            stacks,
                            stack_scaling,
                            ..
                        },
                    ) = event.outcome.as_ref()
                    else {
                        warn!(
                            effect = ?effect_id,
                            "Tick periodico senza dati di scalatura: effetto ignorato."
                        );
                        continue;
                    };

                    pending
                        .resolve_and_enqueue_periodic_from_database_with_generation(
                            &database,
                            effect_id,
                            Some(subscriber),
                            target_selection,
                            &source_attributes,
                            *stacks,
                            stack_scaling.clone(),
                            next_generation,
                        )
                } else {
                    pending
                        .resolve_and_enqueue_from_database_with_generation(
                            &database,
                            effect_id,
                            Some(subscriber),
                            target_selection,
                            &source_attributes,
                            next_generation,
                        )
                };

            if let Err(error) = enqueue_result {
                warn!(
                    "Impossibile accodare l'effetto reattivo {}: {}",
                    effect_id.as_str(),
                    error
                );
            }
        }
    }
}

#[cfg(test)]
mod tests;
