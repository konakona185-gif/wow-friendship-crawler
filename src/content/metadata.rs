use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::{Display, Formatter};

use super::ContentId;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentMetadata {
    pub id: ContentId,
    pub pack_id: ContentId,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl ContentMetadata {
    pub fn validate(&self) -> Result<(), ContentMetadataError> {
        if self.pack_id.namespace() != "pack" {
            return Err(ContentMetadataError::InvalidPackNamespace);
        }

        if self.id.namespace() != self.pack_id.local_id() {
            return Err(ContentMetadataError::MismatchedContentNamespace);
        }

        if self.name.trim().is_empty() {
            return Err(ContentMetadataError::EmptyName);
        }

        let mut tags = HashSet::new();

        for tag in &self.tags {
            if tag.trim().is_empty() {
                return Err(ContentMetadataError::EmptyTag);
            }

            if !tags.insert(tag) {
                return Err(ContentMetadataError::DuplicateTag);
            }
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContentMetadataError {
    InvalidPackNamespace,
    MismatchedContentNamespace,
    EmptyName,
    EmptyTag,
    DuplicateTag,
}

impl Display for ContentMetadataError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InvalidPackNamespace => {
                "Il pacchetto deve usare il namespace 'pack'"
            }
            Self::MismatchedContentNamespace => {
                "Il namespace del contenuto deve corrispondere al pacchetto"
            }
            Self::EmptyName => {
                "Il nome del contenuto non può essere vuoto"
            }
            Self::EmptyTag => {
                "I tag non possono essere vuoti"
            }
            Self::DuplicateTag => {
                "Il contenuto contiene tag duplicati"
            }
        };

        formatter.write_str(message)
    }
}

impl std::error::Error for ContentMetadataError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_metadata() -> ContentMetadata {
        ContentMetadata {
            id: ContentId::new("classic_1_12_1:warrior.charge").unwrap(),
            pack_id: ContentId::new("pack:classic_1_12_1").unwrap(),
            name: "Charge".to_string(),
            description: "Carica verso il bersaglio".to_string(),
            icon: None,
            tags: vec![
                "warrior".to_string(),
                "movement".to_string(),
            ],
        }
    }

    #[test]
    fn valid_metadata_passes_validation() {
        assert!(valid_metadata().validate().is_ok());
    }

    #[test]
    fn metadata_rejects_invalid_pack_namespace() {
        let mut metadata = valid_metadata();
        metadata.pack_id = ContentId::new("classic:content").unwrap();

        assert_eq!(
            metadata.validate(),
            Err(ContentMetadataError::InvalidPackNamespace)
        );
    }

    #[test]
    fn metadata_rejects_mismatched_content_namespace() {
        let mut metadata = valid_metadata();
        metadata.id = ContentId::new("custom:shadow_assault").unwrap();

        assert_eq!(
            metadata.validate(),
            Err(ContentMetadataError::MismatchedContentNamespace)
        );
    }

    #[test]
    fn metadata_rejects_empty_name() {
        let mut metadata = valid_metadata();
        metadata.name = " ".to_string();

        assert_eq!(
            metadata.validate(),
            Err(ContentMetadataError::EmptyName)
        );
    }

    #[test]
    fn metadata_rejects_empty_and_duplicate_tags() {
        let mut metadata = valid_metadata();
        metadata.tags.push(" ".to_string());

        assert_eq!(
            metadata.validate(),
            Err(ContentMetadataError::EmptyTag)
        );

        let mut metadata = valid_metadata();
        metadata.tags.push("warrior".to_string());

        assert_eq!(
            metadata.validate(),
            Err(ContentMetadataError::DuplicateTag)
        );
    }

    #[test]
    fn metadata_supports_ron_serialization() {
        let original = valid_metadata();

        let serialized = ron::to_string(&original).unwrap();
        let restored: ContentMetadata =
            ron::from_str(&serialized).unwrap();

        assert_eq!(original, restored);
        assert!(restored.validate().is_ok());
    }
}
