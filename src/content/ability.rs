use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::{Display, Formatter};

use super::{ContentId, ContentMetadata, ContentMetadataError};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AbilityDefinition {
    pub metadata: ContentMetadata,
    pub activation: AbilityActivation,
    pub targeting: AbilityTargeting,
    pub range: AbilityRange,
    pub cooldown_seconds: f32,
    #[serde(default)]
    pub costs: Vec<ResourceCost>,
    pub effects: Vec<ContentId>,
    #[serde(default)]
    pub class_restrictions: Vec<ContentId>,
    #[serde(default)]
    pub prerequisites: Vec<ContentId>,
}

impl AbilityDefinition {
    pub fn validate(&self) -> Result<(), AbilityValidationError> {
        self.metadata
            .validate()
            .map_err(AbilityValidationError::InvalidMetadata)?;

        if !self.cooldown_seconds.is_finite() || self.cooldown_seconds < 0.0 {
            return Err(AbilityValidationError::InvalidCooldown);
        }

        self.activation.validate()?;
        self.range.validate()?;

        if matches!(
            self.targeting,
            AbilityTargeting::None | AbilityTargeting::SelfOnly
        ) && (self.range.min_range != 0.0 || self.range.max_range != 0.0)
        {
            return Err(AbilityValidationError::InvalidRangeForTargeting);
        }

        if matches!(self.activation, AbilityActivation::Passive) {
            if !self.costs.is_empty() {
                return Err(AbilityValidationError::PassiveHasCosts);
            }

            if self.cooldown_seconds != 0.0 {
                return Err(AbilityValidationError::PassiveHasCooldown);
            }

            if !matches!(self.targeting, AbilityTargeting::None) {
                return Err(AbilityValidationError::PassiveRequiresNoTarget);
            }
        }

        if self.effects.is_empty() {
            return Err(AbilityValidationError::EmptyEffects);
        }

        let mut effects = HashSet::new();

        for effect in &self.effects {
            if !effects.insert(effect) {
                return Err(AbilityValidationError::DuplicateEffect);
            }
        }

        let mut resources = HashSet::new();

        for cost in &self.costs {
            cost.validate()?;

            if !resources.insert(&cost.resource) {
                return Err(AbilityValidationError::DuplicateResourceCost);
            }
        }

        let mut classes = HashSet::new();

        for class in &self.class_restrictions {
            if !classes.insert(class) {
                return Err(AbilityValidationError::DuplicateClassRestriction);
            }
        }

        let mut prerequisites = HashSet::new();

        for prerequisite in &self.prerequisites {
            if prerequisite == &self.metadata.id {
                return Err(AbilityValidationError::SelfPrerequisite);
            }

            if !prerequisites.insert(prerequisite) {
                return Err(AbilityValidationError::DuplicatePrerequisite);
            }
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "parameters", rename_all = "snake_case")]
pub enum AbilityActivation {
    Instant,
    Cast {
        cast_time_seconds: f32,
    },
    Channeled {
        duration_seconds: f32,
        tick_interval_seconds: f32,
    },
    Passive,
}

impl AbilityActivation {
    fn validate(&self) -> Result<(), AbilityValidationError> {
        match self {
            Self::Instant | Self::Passive => Ok(()),
            Self::Cast { cast_time_seconds } => {
                if !cast_time_seconds.is_finite() || *cast_time_seconds <= 0.0 {
                    return Err(AbilityValidationError::InvalidCastTime);
                }

                Ok(())
            }
            Self::Channeled {
                duration_seconds,
                tick_interval_seconds,
            } => {
                if !duration_seconds.is_finite() || *duration_seconds <= 0.0 {
                    return Err(AbilityValidationError::InvalidChannelDuration);
                }

                if !tick_interval_seconds.is_finite()
                    || *tick_interval_seconds <= 0.0
                    || *tick_interval_seconds > *duration_seconds
                {
                    return Err(AbilityValidationError::InvalidChannelTickInterval);
                }

                Ok(())
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AbilityTargeting {
    None,
    SelfOnly,
    FriendlyUnit,
    HostileUnit,
    AnyUnit,
    GroundLocation,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct AbilityRange {
    pub min_range: f32,
    pub max_range: f32,
}

impl AbilityRange {
    pub fn validate(&self) -> Result<(), AbilityValidationError> {
        if !self.min_range.is_finite()
            || !self.max_range.is_finite()
            || self.min_range < 0.0
            || self.max_range < self.min_range
        {
            return Err(AbilityValidationError::InvalidRange);
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ResourceCost {
    pub resource: ContentId,
    pub amount: f32,
}

impl ResourceCost {
    pub fn validate(&self) -> Result<(), AbilityValidationError> {
        if !self.amount.is_finite() || self.amount <= 0.0 {
            return Err(AbilityValidationError::InvalidResourceCost);
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AbilityValidationError {
    InvalidMetadata(ContentMetadataError),
    InvalidCooldown,
    InvalidCastTime,
    InvalidChannelDuration,
    InvalidChannelTickInterval,
    InvalidRange,
    InvalidRangeForTargeting,
    InvalidResourceCost,
    DuplicateResourceCost,
    EmptyEffects,
    DuplicateEffect,
    DuplicateClassRestriction,
    DuplicatePrerequisite,
    SelfPrerequisite,
    PassiveHasCosts,
    PassiveHasCooldown,
    PassiveRequiresNoTarget,
}

impl Display for AbilityValidationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InvalidMetadata(error) => {
                return write!(formatter, "Metadati dell'abilità non validi: {error}");
            }
            Self::InvalidCooldown => {
                "Il cooldown deve essere finito e non negativo"
            }
            Self::InvalidCastTime => {
                "Il tempo di lancio deve essere finito e positivo"
            }
            Self::InvalidChannelDuration => {
                "La durata della canalizzazione deve essere finita e positiva"
            }
            Self::InvalidChannelTickInterval => {
                "L'intervallo dei tick deve essere positivo e non superare la durata"
            }
            Self::InvalidRange => {
                "La gittata deve essere finita, non negativa e coerente"
            }
            Self::InvalidRangeForTargeting => {
                "Le abilità senza bersaglio o su sé stessi devono avere gittata zero"
            }
            Self::InvalidResourceCost => {
                "Il costo della risorsa deve essere finito e positivo"
            }
            Self::DuplicateResourceCost => {
                "L'abilità contiene costi duplicati per la stessa risorsa"
            }
            Self::EmptyEffects => {
                "L'abilità deve avere almeno un effetto"
            }
            Self::DuplicateEffect => {
                "L'abilità contiene riferimenti a effetti duplicati"
            }
            Self::DuplicateClassRestriction => {
                "L'abilità contiene restrizioni di classe duplicate"
            }
            Self::DuplicatePrerequisite => {
                "L'abilità contiene prerequisiti duplicati"
            }
            Self::SelfPrerequisite => {
                "Un'abilità non può essere prerequisito di sé stessa"
            }
            Self::PassiveHasCosts => {
                "Un'abilità passiva non può avere costi di risorse"
            }
            Self::PassiveHasCooldown => {
                "Un'abilità passiva non può avere cooldown"
            }
            Self::PassiveRequiresNoTarget => {
                "Un'abilità passiva non può richiedere un bersaglio"
            }
        };

        formatter.write_str(message)
    }
}

impl std::error::Error for AbilityValidationError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_ability() -> AbilityDefinition {
        AbilityDefinition {
            metadata: ContentMetadata {
                id: ContentId::new("classic_1_12_1:warrior.charge").unwrap(),
                pack_id: ContentId::new("pack:classic_1_12_1").unwrap(),
                name: "Charge".to_string(),
                description: "Carica verso il bersaglio".to_string(),
                icon: None,
                tags: vec!["warrior".to_string()],
            },
            activation: AbilityActivation::Instant,
            targeting: AbilityTargeting::HostileUnit,
            range: AbilityRange {
                min_range: 8.0,
                max_range: 25.0,
            },
            cooldown_seconds: 15.0,
            costs: Vec::new(),
            effects: vec![
                ContentId::new("classic_1_12_1:effect.charge").unwrap(),
            ],
            class_restrictions: vec![
                ContentId::new("classic_1_12_1:class.warrior").unwrap(),
            ],
            prerequisites: Vec::new(),
        }
    }

    #[test]
    fn valid_ability_passes_validation() {
        assert!(valid_ability().validate().is_ok());
    }

    #[test]
    fn ability_rejects_invalid_cooldown_and_range() {
        let mut ability = valid_ability();
        ability.cooldown_seconds = f32::NAN;

        assert_eq!(
            ability.validate(),
            Err(AbilityValidationError::InvalidCooldown)
        );

        let mut ability = valid_ability();
        ability.range.min_range = 30.0;

        assert_eq!(
            ability.validate(),
            Err(AbilityValidationError::InvalidRange)
        );
    }

    #[test]
    fn ability_rejects_invalid_cast_time() {
        let mut ability = valid_ability();
        ability.activation = AbilityActivation::Cast {
            cast_time_seconds: 0.0,
        };

        assert_eq!(
            ability.validate(),
            Err(AbilityValidationError::InvalidCastTime)
        );
    }

    #[test]
    fn ability_rejects_invalid_channel_timing() {
        let mut ability = valid_ability();
        ability.activation = AbilityActivation::Channeled {
            duration_seconds: 3.0,
            tick_interval_seconds: 4.0,
        };

        assert_eq!(
            ability.validate(),
            Err(AbilityValidationError::InvalidChannelTickInterval)
        );
    }

    #[test]
    fn ability_rejects_invalid_and_duplicate_resource_costs() {
        let mut ability = valid_ability();
        ability.costs = vec![
            ResourceCost {
                resource: ContentId::new("classic_1_12_1:resource.mana").unwrap(),
                amount: 0.0,
            },
        ];

        assert_eq!(
            ability.validate(),
            Err(AbilityValidationError::InvalidResourceCost)
        );

        let mut ability = valid_ability();
        let resource = ContentId::new("classic_1_12_1:resource.mana").unwrap();

        ability.costs = vec![
            ResourceCost {
                resource: resource.clone(),
                amount: 10.0,
            },
            ResourceCost {
                resource,
                amount: 5.0,
            },
        ];

        assert_eq!(
            ability.validate(),
            Err(AbilityValidationError::DuplicateResourceCost)
        );
    }

    #[test]
    fn passive_ability_rejects_costs_cooldown_and_targeting() {
        let mut ability = valid_ability();
        ability.activation = AbilityActivation::Passive;
        ability.targeting = AbilityTargeting::None;
        ability.range = AbilityRange {
            min_range: 0.0,
            max_range: 0.0,
        };
        ability.cooldown_seconds = 0.0;
        ability.costs.clear();

        assert!(ability.validate().is_ok());

        ability.costs.push(ResourceCost {
            resource: ContentId::new("classic_1_12_1:resource.mana").unwrap(),
            amount: 10.0,
        });

        assert_eq!(
            ability.validate(),
            Err(AbilityValidationError::PassiveHasCosts)
        );
    }

    #[test]
    fn ability_rejects_missing_and_duplicate_effects() {
        let mut ability = valid_ability();
        ability.effects.clear();

        assert_eq!(
            ability.validate(),
            Err(AbilityValidationError::EmptyEffects)
        );

        let mut ability = valid_ability();
        ability.effects.push(ability.effects[0].clone());

        assert_eq!(
            ability.validate(),
            Err(AbilityValidationError::DuplicateEffect)
        );
    }

    #[test]
    fn ability_rejects_self_prerequisite() {
        let mut ability = valid_ability();
        ability.prerequisites.push(ability.metadata.id.clone());

        assert_eq!(
            ability.validate(),
            Err(AbilityValidationError::SelfPrerequisite)
        );
    }

    #[test]
    fn ability_supports_ron_serialization() {
        let original = valid_ability();

        let serialized = ron::to_string(&original).unwrap();
        let restored: AbilityDefinition = ron::from_str(&serialized).unwrap();

        assert_eq!(original, restored);
        assert!(restored.validate().is_ok());
    }
}
