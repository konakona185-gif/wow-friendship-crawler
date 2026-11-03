use std::collections::VecDeque;
use std::fmt::{Display, Formatter};

use bevy::prelude::{Component, Entity, Resource};

use crate::content::{ContentId, EffectTrigger};

use super::{
    DamageResolution,
    DamageResolutionError,
    HealingResolution,
    HealingResolutionError,
    ResourceResolution,
    ResourceResolutionError,
};

pub const MAX_REACTIVE_EFFECT_GENERATIONS: u8 = 16;

#[derive(Clone, Debug, PartialEq)]
pub enum CombatEventOutcome {
    Damage(DamageResolution),
    Healing(HealingResolution),
    Resource(ResourceResolution),
    AuraExpired {
        aura: ContentId,
        stacks: u32,
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
            Some(CombatEventOutcome::AuraExpired { stacks, .. }) => {
                if *stacks == 0 {
                    return Err(
                        CombatEventContextError::ZeroExpiredAuraStacks,
                    );
                }

                if self.target.is_none() {
                    return Err(
                        CombatEventContextError::MissingAuraExpirationTarget,
                    );
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
    pub fn enqueue(
        &mut self,
        event: CombatEventContext,
    ) -> Result<(), CombatEventContextError> {
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
    ZeroExpiredAuraStacks,
    MissingAuraExpirationTarget,
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
            Self::ZeroExpiredAuraStacks => {
                formatter.write_str(
                    "Un evento di scadenza deve contenere almeno uno stack",
                )
            }
            Self::MissingAuraExpirationTarget => {
                formatter.write_str(
                    "Un evento di scadenza richiede il bersaglio dell'aura",
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

    fn content_id(value: &str) -> ContentId {
        ContentId::new(
            format!("classic_1_12_1:{value}"),
        )
        .unwrap()
    }

    fn valid_context(trigger: EffectTrigger) -> CombatEventContext {
        CombatEventContext {
            trigger,
            source: Some(source()),
            target: Some(target()),
            ability: Some(
                content_id("ability.test"),
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
            school: content_id("school.physical"),
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
            resource: content_id("resource.mana"),
            operation: super::super::ResourceOperation::Gain,
            base_amount: 50.0,
        };

        CombatEventOutcome::Resource(
            ResourceResolution::new(request, 30.0).unwrap(),
        )
    }

    fn aura_expired_outcome(stacks: u32) -> CombatEventOutcome {
        CombatEventOutcome::AuraExpired {
            aura: content_id("aura.test"),
            stacks,
        }
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
    fn event_context_validates_aura_expiration_outcomes() {
        let mut context = valid_context(EffectTrigger::OnExpire);
        context.outcome = Some(aura_expired_outcome(2));

        assert!(context.validate().is_ok());
    }

    #[test]
    fn event_context_rejects_zero_expired_aura_stacks() {
        let mut context = valid_context(EffectTrigger::OnExpire);
        context.outcome = Some(aura_expired_outcome(0));

        assert_eq!(
            context.validate(),
            Err(CombatEventContextError::ZeroExpiredAuraStacks)
        );
    }

    #[test]
    fn event_context_rejects_aura_expiration_without_target() {
        let mut context = valid_context(EffectTrigger::OnExpire);
        context.target = None;
        context.outcome = Some(aura_expired_outcome(1));

        assert_eq!(
            context.validate(),
            Err(
                CombatEventContextError::MissingAuraExpirationTarget
            )
        );
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
        first.effect = Some(content_id("effect.first"));

        let mut second = valid_context(EffectTrigger::OnDamageDealt);
        second.effect = Some(content_id("effect.second"));

        queue.enqueue(first.clone()).unwrap();
        queue.enqueue(second.clone()).unwrap();

        assert_eq!(queue.len(), 2);

        let drained = queue.drain();

        assert_eq!(drained, vec![first, second]);
        assert!(queue.is_empty());
    }

    #[test]
    fn event_queue_preserves_generation_metadata() {
        let mut queue = CombatEventQueue::default();

        let first = valid_context(EffectTrigger::OnCast);
        let second = valid_context(EffectTrigger::OnDamageDealt);

        queue
            .enqueue_with_generation(first.clone(), 0)
            .unwrap();
        queue
            .enqueue_with_generation(second.clone(), 7)
            .unwrap();

        let drained = queue.drain_with_generation();

        assert_eq!(
            drained,
            vec![
                (first, 0),
                (second, 7),
            ]
        );
        assert!(queue.is_empty());
    }

    #[test]
    fn event_queue_uses_generation_zero_for_standard_enqueue() {
        let mut queue = CombatEventQueue::default();
        let event = valid_context(EffectTrigger::OnCast);

        queue.enqueue(event.clone()).unwrap();

        assert_eq!(
            queue.drain_with_generation(),
            vec![(event, 0)]
        );
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

    #[test]
    fn effect_subscriptions_start_empty() {
        let subscriptions = CombatEffectSubscriptions::default();

        assert!(subscriptions.is_empty());
        assert_eq!(subscriptions.len(), 0);
    }

    #[test]
    fn effect_subscriptions_reject_duplicates() {
        let mut subscriptions = CombatEffectSubscriptions::default();
        let effect = content_id("effect.passive");

        assert!(subscriptions.subscribe(effect.clone()));
        assert!(!subscriptions.subscribe(effect.clone()));

        assert_eq!(subscriptions.len(), 1);
        assert!(subscriptions.contains(&effect));
    }

    #[test]
    fn effect_subscriptions_preserve_registration_order() {
        let mut subscriptions = CombatEffectSubscriptions::default();

        let first = content_id("effect.first");
        let second = content_id("effect.second");

        assert!(subscriptions.subscribe(first.clone()));
        assert!(subscriptions.subscribe(second.clone()));

        assert_eq!(
            subscriptions.iter().cloned().collect::<Vec<_>>(),
            vec![first, second]
        );
    }

    #[test]
    fn effect_subscriptions_remove_only_requested_effect() {
        let mut subscriptions = CombatEffectSubscriptions::default();

        let first = content_id("effect.first");
        let second = content_id("effect.second");

        subscriptions.subscribe(first.clone());
        subscriptions.subscribe(second.clone());

        assert!(subscriptions.unsubscribe(&first));
        assert!(!subscriptions.unsubscribe(&first));

        assert_eq!(subscriptions.len(), 1);
        assert!(!subscriptions.contains(&first));
        assert!(subscriptions.contains(&second));
    }
}
