use std::collections::VecDeque;
use std::fmt::{Display, Formatter};

use bevy::prelude::{Component, Entity, Resource};

use crate::content::{ContentId, EffectTrigger, PeriodicEffectStackScaling};

use crate::combat::{
    DamageResolution, DamageResolutionError, HealingResolution, HealingResolutionError,
    ResourceResolution, ResourceResolutionError,
};

pub const MAX_REACTIVE_EFFECT_GENERATIONS: u8 = 16;

#[derive(Clone, Debug, PartialEq)]
pub enum CombatEventOutcome {
    Damage(DamageResolution),
    Healing(HealingResolution),
    Resource(ResourceResolution),
    AuraApplied {
        aura: ContentId,
        total_stacks: u32,
    },
    AuraRemoved {
        aura: ContentId,
        removed_stacks: u32,
    },
    AuraExpired {
        aura: ContentId,
        stacks: u32,
    },
    PeriodicTick {
        aura: ContentId,
        stacks: u32,
        stack_scaling: PeriodicEffectStackScaling,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct CombatEventContext {
    pub trigger: EffectTrigger,
    pub source: Option<Entity>,
    pub target: Option<Entity>,
    pub ability: Option<ContentId>,
    pub effect: Option<ContentId>,
    pub outcome: Option<CombatEventOutcome>,
}

impl CombatEventContext {
    pub fn validate(&self) -> Result<(), CombatEventContextError> {
        match &self.outcome {
            Some(CombatEventOutcome::Damage(resolution)) => {
                resolution
                    .validate()
                    .map_err(CombatEventContextError::InvalidDamage)?;

                self.validate_participants(resolution.request.source, resolution.request.target)?;
            }
            Some(CombatEventOutcome::Healing(resolution)) => {
                resolution
                    .validate()
                    .map_err(CombatEventContextError::InvalidHealing)?;

                self.validate_participants(resolution.request.source, resolution.request.target)?;
            }
            Some(CombatEventOutcome::Resource(resolution)) => {
                resolution
                    .validate()
                    .map_err(CombatEventContextError::InvalidResource)?;

                self.validate_participants(resolution.request.source, resolution.request.target)?;
            }
            Some(CombatEventOutcome::AuraApplied { total_stacks, .. }) => {
                if self.trigger != EffectTrigger::OnAuraApplied {
                    return Err(CombatEventContextError::AuraApplicationTriggerMismatch);
                }

                if *total_stacks == 0 {
                    return Err(CombatEventContextError::ZeroAppliedAuraStacks);
                }

                if self.target.is_none() {
                    return Err(CombatEventContextError::MissingAuraApplicationTarget);
                }
            }
            Some(CombatEventOutcome::AuraRemoved { removed_stacks, .. }) => {
                if self.trigger != EffectTrigger::OnAuraRemoved {
                    return Err(CombatEventContextError::AuraRemovalTriggerMismatch);
                }

                if *removed_stacks == 0 {
                    return Err(CombatEventContextError::ZeroRemovedAuraStacks);
                }

                if self.target.is_none() {
                    return Err(CombatEventContextError::MissingAuraRemovalTarget);
                }
            }
            Some(CombatEventOutcome::AuraExpired { stacks, .. }) => {
                if self.trigger != EffectTrigger::OnExpire {
                    return Err(CombatEventContextError::AuraExpirationTriggerMismatch);
                }

                if *stacks == 0 {
                    return Err(CombatEventContextError::ZeroExpiredAuraStacks);
                }

                if self.target.is_none() {
                    return Err(CombatEventContextError::MissingAuraExpirationTarget);
                }
            }
            Some(CombatEventOutcome::PeriodicTick { stacks, .. }) => {
                if self.trigger != EffectTrigger::OnPeriodicTick {
                    return Err(CombatEventContextError::PeriodicTickTriggerMismatch);
                }

                if *stacks == 0 {
                    return Err(CombatEventContextError::ZeroPeriodicTickStacks);
                }

                if self.target.is_none() {
                    return Err(CombatEventContextError::MissingPeriodicTickTarget);
                }

                if self.effect.is_none() {
                    return Err(CombatEventContextError::MissingPeriodicTickEffect);
                }
            }
            None => {}
        }

        Ok(())
    }

    fn validate_participants(
        &self,
        source: Option<Entity>,
        target: Entity,
    ) -> Result<(), CombatEventContextError> {
        if self.source != source {
            return Err(CombatEventContextError::SourceMismatch);
        }

        if self.target != Some(target) {
            return Err(CombatEventContextError::TargetMismatch);
        }

        Ok(())
    }
}

#[derive(Component, Clone, Debug, Default, PartialEq, Eq)]
pub struct CombatEffectSubscriptions {
    effects: Vec<ContentId>,
}

impl CombatEffectSubscriptions {
    pub fn subscribe(&mut self, effect: ContentId) -> bool {
        if self.effects.contains(&effect) {
            return false;
        }

        self.effects.push(effect);

        true
    }

    pub fn unsubscribe(&mut self, effect: &ContentId) -> bool {
        let Some(index) = self
            .effects
            .iter()
            .position(|registered| registered == effect)
        else {
            return false;
        };

        self.effects.remove(index);

        true
    }

    pub fn contains(&self, effect: &ContentId) -> bool {
        self.effects.contains(effect)
    }

    pub fn iter(&self) -> impl Iterator<Item = &ContentId> {
        self.effects.iter()
    }

    pub fn len(&self) -> usize {
        self.effects.len()
    }

    pub fn is_empty(&self) -> bool {
        self.effects.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq)]
struct QueuedCombatEvent {
    context: CombatEventContext,
    generation: u8,
}

#[derive(Resource, Default)]
pub struct CombatEventQueue {
    pending: VecDeque<QueuedCombatEvent>,
}

impl CombatEventQueue {
    pub fn enqueue(&mut self, event: CombatEventContext) -> Result<(), CombatEventContextError> {
        self.enqueue_with_generation(event, 0)
    }

    pub fn enqueue_with_generation(
        &mut self,
        event: CombatEventContext,
        generation: u8,
    ) -> Result<(), CombatEventContextError> {
        event.validate()?;

        self.pending.push_back(QueuedCombatEvent {
            context: event,
            generation,
        });

        Ok(())
    }

    pub fn drain(&mut self) -> Vec<CombatEventContext> {
        self.drain_with_generation()
            .into_iter()
            .map(|(event, _)| event)
            .collect()
    }

    pub fn drain_with_generation(&mut self) -> Vec<(CombatEventContext, u8)> {
        self.pending
            .drain(..)
            .map(|queued| (queued.context, queued.generation))
            .collect()
    }

    pub fn len(&self) -> usize {
        self.pending.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CombatEventContextError {
    InvalidDamage(DamageResolutionError),
    InvalidHealing(HealingResolutionError),
    InvalidResource(ResourceResolutionError),
    SourceMismatch,
    TargetMismatch,
    AuraApplicationTriggerMismatch,
    ZeroAppliedAuraStacks,
    MissingAuraApplicationTarget,
    AuraRemovalTriggerMismatch,
    ZeroRemovedAuraStacks,
    MissingAuraRemovalTarget,
    AuraExpirationTriggerMismatch,
    ZeroExpiredAuraStacks,
    MissingAuraExpirationTarget,
    PeriodicTickTriggerMismatch,
    ZeroPeriodicTickStacks,
    MissingPeriodicTickTarget,
    MissingPeriodicTickEffect,
}

impl Display for CombatEventContextError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidDamage(error) => {
                write!(formatter, "Risultato del danno non valido: {error}")
            }
            Self::InvalidHealing(error) => {
                write!(formatter, "Risultato della guarigione non valido: {error}")
            }
            Self::InvalidResource(error) => {
                write!(formatter, "Risultato della risorsa non valido: {error}")
            }
            Self::SourceMismatch => {
                formatter.write_str("La sorgente dell'evento non corrisponde al risultato")
            }
            Self::TargetMismatch => {
                formatter.write_str("Il bersaglio dell'evento non corrisponde al risultato")
            }
            Self::AuraApplicationTriggerMismatch => {
                formatter.write_str("Un evento di applicazione richiede il trigger OnAuraApplied")
            }
            Self::ZeroAppliedAuraStacks => {
                formatter.write_str("Un evento di applicazione deve contenere almeno uno stack")
            }
            Self::MissingAuraApplicationTarget => {
                formatter.write_str("Un evento di applicazione richiede il bersaglio dell'aura")
            }
            Self::AuraRemovalTriggerMismatch => {
                formatter.write_str("Un evento di rimozione richiede il trigger OnAuraRemoved")
            }
            Self::ZeroRemovedAuraStacks => formatter
                .write_str("Un evento di rimozione deve contenere almeno uno stack rimosso"),
            Self::MissingAuraRemovalTarget => {
                formatter.write_str("Un evento di rimozione richiede il bersaglio dell'aura")
            }
            Self::AuraExpirationTriggerMismatch => {
                formatter.write_str("Un evento di scadenza richiede il trigger OnExpire")
            }
            Self::ZeroExpiredAuraStacks => {
                formatter.write_str("Un evento di scadenza deve contenere almeno uno stack")
            }
            Self::MissingAuraExpirationTarget => {
                formatter.write_str("Un evento di scadenza richiede il bersaglio dell'aura")
            }
            Self::PeriodicTickTriggerMismatch => {
                formatter.write_str("Un evento periodico richiede il trigger OnPeriodicTick")
            }
            Self::ZeroPeriodicTickStacks => {
                formatter.write_str("Un tick periodico deve contenere almeno uno stack")
            }
            Self::MissingPeriodicTickTarget => {
                formatter.write_str("Un tick periodico richiede il bersaglio dell'aura")
            }
            Self::MissingPeriodicTickEffect => {
                formatter.write_str("Un tick periodico richiede l'ID del suo effetto")
            }
        }
    }
}

impl std::error::Error for CombatEventContextError {}

#[cfg(test)]
mod tests;
