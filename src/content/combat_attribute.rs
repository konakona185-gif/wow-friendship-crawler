use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

use super::{ContentMetadata, ContentMetadataError};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CombatAttributeDefinition {
    pub metadata: ContentMetadata,
    pub value_kind: CombatAttributeValueKind,
}

impl CombatAttributeDefinition {
    pub fn validate(&self) -> Result<(), CombatAttributeValidationError> {
        self.metadata
            .validate()
            .map_err(CombatAttributeValidationError::InvalidMetadata)?;

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CombatAttributeValueKind {
    Flat,
    PercentageFraction,
    Rating,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CombatAttributeValidationError {
    InvalidMetadata(ContentMetadataError),
}

impl Display for CombatAttributeValidationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidMetadata(error) => {
                write!(
                    formatter,
                    "Metadati dell'attributo di combattimento non validi: {error}"
                )
            }
        }
    }
}

impl std::error::Error for CombatAttributeValidationError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::ContentId;

    fn valid_attribute() -> CombatAttributeDefinition {
        CombatAttributeDefinition {
            metadata: ContentMetadata {
                id: ContentId::new("classic_1_12_1:stat.armor").unwrap(),
                pack_id: ContentId::new("pack:classic_1_12_1").unwrap(),
                name: "Armor".to_string(),
                description: String::new(),
                icon: None,
                tags: Vec::new(),
            },
            value_kind: CombatAttributeValueKind::Flat,
        }
    }

    #[test]
    fn valid_combat_attribute_passes_validation() {
        assert!(valid_attribute().validate().is_ok());
    }

    #[test]
    fn combat_attribute_rejects_invalid_metadata() {
        let mut attribute = valid_attribute();
        attribute.metadata.name.clear();

        assert_eq!(
            attribute.validate(),
            Err(CombatAttributeValidationError::InvalidMetadata(
                ContentMetadataError::EmptyName
            ))
        );
    }

    #[test]
    fn combat_attribute_supports_ron_serialization() {
        for value_kind in [
            CombatAttributeValueKind::Flat,
            CombatAttributeValueKind::PercentageFraction,
            CombatAttributeValueKind::Rating,
        ] {
            let mut original = valid_attribute();
            original.value_kind = value_kind;

            let serialized = ron::to_string(&original).unwrap();
            let restored: CombatAttributeDefinition = ron::from_str(&serialized).unwrap();

            assert_eq!(original, restored);
            assert!(restored.validate().is_ok());
        }
    }
}
