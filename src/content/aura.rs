use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

use super::{ContentId, ContentMetadata, ContentMetadataError};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AuraDefinition {
    pub metadata: ContentMetadata,
    pub max_stacks: u32,
    pub stacking_policy: AuraStackingPolicy,
    #[serde(default)]
    pub stacking_scope: AuraStackingScope,
    #[serde(default)]
    pub refresh_policy: AuraRefreshPolicy,
    #[serde(default)]
    pub periodic_effects: Vec<PeriodicAuraEffect>,
}

impl AuraDefinition {
    pub fn validate(&self) -> Result<(), AuraValidationError> {
        self.metadata
            .validate()
            .map_err(AuraValidationError::InvalidMetadata)?;

        if self.max_stacks == 0 {
            return Err(AuraValidationError::ZeroMaxStacks);
        }

        for periodic_effect in &self.periodic_effects {
            if !periodic_effect.interval_seconds.is_finite()
                || periodic_effect.interval_seconds <= 0.0
            {
                return Err(AuraValidationError::InvalidPeriodicEffectInterval);
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PeriodicAuraEffect {
    pub effect: ContentId,
    pub interval_seconds: f32,
    #[serde(default)]
    pub stack_scaling: PeriodicEffectStackScaling,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuraStackingScope {
    #[default]
    Shared,
    PerCaster,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuraRefreshPolicy {
    #[default]
    ResetDuration,
    Pandemic,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PeriodicEffectStackScaling {
    #[default]
    None,
    Linear,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuraStackingPolicy {
    RefreshDuration,
    AddStacks,
    Replace,
    Independent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuraValidationError {
    InvalidMetadata(ContentMetadataError),
    ZeroMaxStacks,
    InvalidPeriodicEffectInterval,
}

impl Display for AuraValidationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InvalidMetadata(error) => {
                return write!(formatter, "Metadati dell'aura non validi: {error}");
            }
            Self::ZeroMaxStacks => "Il numero massimo di stack deve essere maggiore di zero",
            Self::InvalidPeriodicEffectInterval => {
                "L'intervallo dell'effetto periodico deve essere finito e positivo"
            }
        };

        formatter.write_str(message)
    }
}

impl std::error::Error for AuraValidationError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::ContentId;

    fn valid_aura() -> AuraDefinition {
        AuraDefinition {
            metadata: ContentMetadata {
                id: ContentId::new("classic_1_12_1:aura.test").unwrap(),
                pack_id: ContentId::new("pack:classic_1_12_1").unwrap(),
                name: "Test Aura".to_string(),
                description: String::new(),
                icon: None,
                tags: Vec::new(),
            },
            max_stacks: 5,
            stacking_policy: AuraStackingPolicy::AddStacks,
            stacking_scope: AuraStackingScope::Shared,
            refresh_policy: AuraRefreshPolicy::ResetDuration,
            periodic_effects: Vec::new(),
        }
    }

    #[test]
    fn valid_aura_passes_validation() {
        assert!(valid_aura().validate().is_ok());
    }

    #[test]
    fn aura_rejects_zero_max_stacks() {
        let mut aura = valid_aura();
        aura.max_stacks = 0;

        assert_eq!(aura.validate(), Err(AuraValidationError::ZeroMaxStacks));
    }

    #[test]
    fn aura_rejects_invalid_periodic_effect_intervals() {
        let mut aura = valid_aura();

        aura.periodic_effects.push(PeriodicAuraEffect {
            effect: ContentId::new("classic_1_12_1:effect.periodic_test").unwrap(),
            interval_seconds: 0.0,
            stack_scaling: PeriodicEffectStackScaling::None,
        });

        assert_eq!(
            aura.validate(),
            Err(AuraValidationError::InvalidPeriodicEffectInterval)
        );

        aura.periodic_effects[0].interval_seconds = f32::INFINITY;

        assert_eq!(
            aura.validate(),
            Err(AuraValidationError::InvalidPeriodicEffectInterval)
        );
    }

    #[test]
    fn aura_rejects_invalid_metadata() {
        let mut aura = valid_aura();
        aura.metadata.name.clear();

        assert_eq!(
            aura.validate(),
            Err(AuraValidationError::InvalidMetadata(
                ContentMetadataError::EmptyName
            ))
        );
    }

    #[test]
    fn aura_supports_ron_serialization() {
        let mut original = valid_aura();

        original.stacking_scope = AuraStackingScope::PerCaster;
        original.refresh_policy = AuraRefreshPolicy::Pandemic;

        original.periodic_effects.push(PeriodicAuraEffect {
            effect: ContentId::new("classic_1_12_1:effect.periodic_test").unwrap(),
            interval_seconds: 3.0,
            stack_scaling: PeriodicEffectStackScaling::Linear,
        });

        let serialized = ron::to_string(&original).unwrap();
        let restored: AuraDefinition = ron::from_str(&serialized).unwrap();

        assert_eq!(original, restored);
        assert!(restored.validate().is_ok());
    }

    #[test]
    fn aura_policies_support_ron_serialization() {
        for scope in [AuraStackingScope::Shared, AuraStackingScope::PerCaster] {
            let serialized = ron::to_string(&scope).unwrap();
            let restored: AuraStackingScope = ron::from_str(&serialized).unwrap();

            assert_eq!(restored, scope);
        }

        for policy in [
            AuraRefreshPolicy::ResetDuration,
            AuraRefreshPolicy::Pandemic,
        ] {
            let serialized = ron::to_string(&policy).unwrap();
            let restored: AuraRefreshPolicy = ron::from_str(&serialized).unwrap();

            assert_eq!(restored, policy);
        }

        for scaling in [
            PeriodicEffectStackScaling::None,
            PeriodicEffectStackScaling::Linear,
        ] {
            let serialized = ron::to_string(&scaling).unwrap();
            let restored: PeriodicEffectStackScaling = ron::from_str(&serialized).unwrap();

            assert_eq!(restored, scaling);
        }
    }

    #[test]
    fn stacking_policies_support_ron_serialization() {
        for policy in [
            AuraStackingPolicy::RefreshDuration,
            AuraStackingPolicy::AddStacks,
            AuraStackingPolicy::Replace,
            AuraStackingPolicy::Independent,
        ] {
            let serialized = ron::to_string(&policy).unwrap();
            let restored: AuraStackingPolicy = ron::from_str(&serialized).unwrap();

            assert_eq!(restored, policy);
        }
    }
}
