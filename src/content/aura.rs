use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

use super::{
    ContentMetadata,
    ContentMetadataError,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AuraDefinition {
    pub metadata: ContentMetadata,
    pub max_stacks: u32,
    pub stacking_policy: AuraStackingPolicy,
}

impl AuraDefinition {
    pub fn validate(&self) -> Result<(), AuraValidationError> {
        self.metadata
            .validate()
            .map_err(AuraValidationError::InvalidMetadata)?;

        if self.max_stacks == 0 {
            return Err(AuraValidationError::ZeroMaxStacks);
        }

        Ok(())
    }
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
}

impl Display for AuraValidationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InvalidMetadata(error) => {
                return write!(
                    formatter,
                    "Metadati dell'aura non validi: {error}"
                );
            }
            Self::ZeroMaxStacks => {
                "Il numero massimo di stack deve essere maggiore di zero"
            }
        };

        formatter.write_str(message)
    }
}

impl std::error::Error for AuraValidationError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_aura() -> AuraDefinition {
        AuraDefinition {
            metadata: ContentMetadata {
                id: ContentId::new(
                    "classic_1_12_1:aura.test"
                )
                .unwrap(),
                pack_id: ContentId::new(
                    "pack:classic_1_12_1"
                )
                .unwrap(),
                name: "Test Aura".to_string(),
                description: String::new(),
                icon: None,
                tags: Vec::new(),
            },
            max_stacks: 5,
            stacking_policy: AuraStackingPolicy::AddStacks,
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

        assert_eq!(
            aura.validate(),
            Err(AuraValidationError::ZeroMaxStacks)
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
        let original = valid_aura();

        let serialized = ron::to_string(&original).unwrap();
        let restored: AuraDefinition =
            ron::from_str(&serialized).unwrap();

        assert_eq!(original, restored);
        assert!(restored.validate().is_ok());
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
            let restored: AuraStackingPolicy =
                ron::from_str(&serialized).unwrap();

            assert_eq!(restored, policy);
        }
    }
}
