use std::collections::VecDeque;
use std::fmt::{Display, Formatter};

use bevy::prelude::{Entity, Resource};

use crate::content::{ContentId, EffectTrigger};

use super::{
    DamageResolution,
    DamageResolutionError,
    HealingResolution,
    HealingResolutionError,
    ResourceResolution,
    ResourceResolutionError,
};

#[derive(Clone, Debug, PartialEq)]
pub enum CombatEventOutcome {
    Damage(DamageResolution),
    Healing(HealingResolution),
    Resource(ResourceResolution),
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

                self.validate_participants(
                    resolution.request.source,
                    resolution.request.target,
                )?;
            }
            Some(CombatEventOutcome::Healing(resolution)) => {
                resolution
                    .validate()
                    .map_err(CombatEventContextError::InvalidHealing)?;

                self.validate_participants(
                    resolution.request.source,
                    resolution.request.target,
                )?;
            }
            Some(CombatEventOutcome::Resource(resolution)) => {
                resolution
                    .validate()
                    .map_err(CombatEventContextError::InvalidResource)?;

                self.validate_participants(
                    resolution.request.source,
                    resolution.request.target,
                )?;
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

#[derive(Resource, Default)]
pub struct CombatEventQueue {
    pending: VecDeque<CombatEventContext>,
}

impl CombatEventQueue {
    pub fn enqueue(
        &mut self,
        event: CombatEventContext,
    ) -> Result<(), CombatEventContextError> {
        event.validate()?;

        self.pending.push_back(event);

        Ok(())
    }

    pub fn drain(&mut self) -> Vec<CombatEventContext> {
        self.pending.drain(..).collect()
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
                formatter.write_str(
                    "La sorgente dell'evento non corrisponde al risultato",
                )
            }
            Self::TargetMismatch => {
                formatter.write_str(
                    "Il bersaglio dell'evento non corrisponde al risultato",
                )
            }
        }
    }
}

impl std::error::Error for CombatEventContextError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn source() -> Entity {
        Entity::from_bits(1)
    }

    fn target() -> Entity {
        Entity::from_bits(2)
    }

    fn valid_context(trigger: EffectTrigger) -> CombatEventContext {
        CombatEventContext {
            trigger,
            source: Some(source()),
            target: Some(target()),
            ability: Some(
                ContentId::new("classic_1_12_1:ability.test").unwrap(),
            ),
            effect: None,
            outcome: None,
        }
    }

    fn damage_outcome() -> CombatEventOutcome {
        let request = super::super::DamageRequest {
            source: Some(source()),
            target: target(),
            base_amount: 100.0,
            school: ContentId::new(
                "classic_1_12_1:school.physical",
            )
            .unwrap(),
        };

        CombatEventOutcome::Damage(
            DamageResolution::new(
                request,
                super::super::DamageMitigationResult::new(
                    100.0,
                    25.0,
                )
                .unwrap(),
            )
            .unwrap(),
        )
    }

    fn healing_outcome() -> CombatEventOutcome {
        let request = super::super::HealingRequest {
            source: Some(source()),
            target: target(),
            base_amount: 100.0,
        };

        CombatEventOutcome::Healing(
            HealingResolution::new(request, 75.0).unwrap(),
        )
    }

    fn resource_outcome() -> CombatEventOutcome {
        let request = super::super::ResourceRequest {
            source: Some(source()),
            target: target(),
            resource: ContentId::new(
                "classic_1_12_1:resource.mana",
            )
            .unwrap(),
            operation: super::super::ResourceOperation::Gain,
            base_amount: 50.0,
        };

        CombatEventOutcome::Resource(
            ResourceResolution::new(request, 30.0).unwrap(),
        )
    }

    #[test]
    fn event_context_without_outcome_passes_validation() {
        let context = valid_context(EffectTrigger::OnCast);

        assert!(context.validate().is_ok());
    }

    #[test]
    fn event_context_validates_damage_outcomes() {
        let mut context = valid_context(EffectTrigger::OnDamageDealt);
        context.outcome = Some(damage_outcome());

        assert!(context.validate().is_ok());
    }

    #[test]
    fn event_context_validates_healing_outcomes() {
        let mut context = valid_context(EffectTrigger::OnCast);
        context.outcome = Some(healing_outcome());

        assert!(context.validate().is_ok());
    }

    #[test]
    fn event_context_validates_resource_outcomes() {
        let mut context = valid_context(EffectTrigger::OnCast);
        context.outcome = Some(resource_outcome());

        assert!(context.validate().is_ok());
    }

    #[test]
    fn event_context_rejects_source_mismatch() {
        let mut context = valid_context(EffectTrigger::OnDamageDealt);

        let mut outcome = damage_outcome();

        if let CombatEventOutcome::Damage(resolution) = &mut outcome {
            resolution.request.source = None;
        }

        context.outcome = Some(outcome);

        assert_eq!(
            context.validate(),
            Err(CombatEventContextError::SourceMismatch)
        );
    }

    #[test]
    fn event_context_rejects_target_mismatch() {
        let mut context = valid_context(EffectTrigger::OnDamageDealt);

        let mut outcome = damage_outcome();

        if let CombatEventOutcome::Damage(resolution) = &mut outcome {
            resolution.request.target = source();
        }

        context.outcome = Some(outcome);

        assert_eq!(
            context.validate(),
            Err(CombatEventContextError::TargetMismatch)
        );
    }

    #[test]
    fn event_context_rejects_invalid_outcomes() {
        let mut context = valid_context(EffectTrigger::OnDamageDealt);

        let mut outcome = damage_outcome();

        if let CombatEventOutcome::Damage(resolution) = &mut outcome {
            resolution.mitigation.mitigated_amount = 150.0;
        }

        context.outcome = Some(outcome);

        assert_eq!(
            context.validate(),
            Err(CombatEventContextError::InvalidDamage(
                DamageResolutionError::InvalidMitigation(
                    super::super::DamageMitigationError::MitigationExceedsIncomingAmount
                )
            ))
        );
    }

    #[test]
    fn event_queue_starts_empty() {
        let queue = CombatEventQueue::default();

        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0);
    }

    #[test]
    fn event_queue_preserves_fifo_order() {
        let mut queue = CombatEventQueue::default();

        let mut first = valid_context(EffectTrigger::OnCast);
        first.effect = Some(
            ContentId::new("classic_1_12_1:effect.first").unwrap(),
        );

        let mut second = valid_context(EffectTrigger::OnDamageDealt);
        second.effect = Some(
            ContentId::new("classic_1_12_1:effect.second").unwrap(),
        );

        queue.enqueue(first.clone()).unwrap();
        queue.enqueue(second.clone()).unwrap();

        assert_eq!(queue.len(), 2);

        let drained = queue.drain();

        assert_eq!(drained, vec![first, second]);
        assert!(queue.is_empty());
    }

    #[test]
    fn event_queue_rejects_invalid_events_without_mutation() {
        let mut queue = CombatEventQueue::default();

        let mut invalid = valid_context(EffectTrigger::OnDamageDealt);
        invalid.source = None;
        invalid.outcome = Some(damage_outcome());

        assert_eq!(
            queue.enqueue(invalid),
            Err(CombatEventContextError::SourceMismatch)
        );

        assert!(queue.is_empty());
    }
}
