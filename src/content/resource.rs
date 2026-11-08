use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

use super::{ContentMetadata, ContentMetadataError};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ResourceDefinition {
    pub metadata: ContentMetadata,
}

impl ResourceDefinition {
    pub fn validate(&self) -> Result<(), ResourceValidationError> {
        self.metadata
            .validate()
            .map_err(ResourceValidationError::InvalidMetadata)?;

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceValidationError {
    InvalidMetadata(ContentMetadataError),
}

impl Display for ResourceValidationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidMetadata(error) => {
                write!(formatter, "Metadati della risorsa non validi: {error}")
            }
        }
    }
}

impl std::error::Error for ResourceValidationError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::ContentId;

    fn valid_resource() -> ResourceDefinition {
        ResourceDefinition {
            metadata: ContentMetadata {
                id: ContentId::new("classic_1_12_1:resource.mana").unwrap(),
                pack_id: ContentId::new("pack:classic_1_12_1").unwrap(),
                name: "Mana".to_string(),
                description: String::new(),
                icon: None,
                tags: Vec::new(),
            },
        }
    }

    #[test]
    fn valid_resource_passes_validation() {
        assert!(valid_resource().validate().is_ok());
    }

    #[test]
    fn resource_rejects_invalid_metadata() {
        let mut resource = valid_resource();
        resource.metadata.name.clear();

        assert_eq!(
            resource.validate(),
            Err(ResourceValidationError::InvalidMetadata(
                ContentMetadataError::EmptyName
            ))
        );
    }

    #[test]
    fn resource_supports_ron_serialization() {
        let original = valid_resource();

        let serialized = ron::to_string(&original).unwrap();
        let restored: ResourceDefinition = ron::from_str(&serialized).unwrap();

        assert_eq!(original, restored);
        assert!(restored.validate().is_ok());
    }
}
