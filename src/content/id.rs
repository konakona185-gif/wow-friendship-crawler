use serde::{Deserialize, Deserializer, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct ContentId(String);

impl ContentId {
    pub fn new(value: impl Into<String>) -> Result<Self, ContentIdError> {
        let value = value.into();
        let mut parts = value.split(':');

        let namespace = parts.next().unwrap_or_default();
        let local_id = parts.next().ok_or(ContentIdError::MissingSeparator)?;

        if parts.next().is_some() {
            return Err(ContentIdError::MultipleSeparators);
        }

        if !is_valid_id_part(namespace) {
            return Err(ContentIdError::InvalidNamespace);
        }

        if !is_valid_id_part(local_id) {
            return Err(ContentIdError::InvalidLocalId);
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn namespace(&self) -> &str {
        self.0
            .split_once(':')
            .map_or("", |(namespace, _)| namespace)
    }

    pub fn local_id(&self) -> &str {
        self.0
            .split_once(':')
            .map_or("", |(_, local_id)| local_id)
    }
}

fn is_valid_id_part(value: &str) -> bool {
    !value.is_empty()
        && value.chars().all(|character| {
            character.is_ascii_lowercase()
                || character.is_ascii_digit()
                || matches!(character, '_' | '-' | '.')
        })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContentIdError {
    MissingSeparator,
    MultipleSeparators,
    InvalidNamespace,
    InvalidLocalId,
}

impl Display for ContentIdError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::MissingSeparator => {
                "L'identificativo deve contenere un namespace separato da ':'"
            }
            Self::MultipleSeparators => {
                "L'identificativo deve contenere un solo separatore ':'"
            }
            Self::InvalidNamespace => {
                "Il namespace contiene caratteri non validi"
            }
            Self::InvalidLocalId => {
                "L'identificativo locale contiene caratteri non validi"
            }
        };

        formatter.write_str(message)
    }
}

impl std::error::Error for ContentIdError {}

impl<'de> Deserialize<'de> for ContentId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_id_splits_namespace_and_local_id() {
        let id = ContentId::new("classic_1_12_1:warrior.charge").unwrap();

        assert_eq!(id.as_str(), "classic_1_12_1:warrior.charge");
        assert_eq!(id.namespace(), "classic_1_12_1");
        assert_eq!(id.local_id(), "warrior.charge");
    }

    #[test]
    fn content_id_rejects_invalid_values() {
        let invalid_ids = [
            "warrior.charge",
            ":warrior.charge",
            "classic:",
            "Classic:Charge",
            "classic:warrior charge",
            "classic:warrior:charge",
        ];

        for value in invalid_ids {
            assert!(
                ContentId::new(value).is_err(),
                "Identificativo non valido accettato: {value}"
            );
        }
    }

    #[test]
    fn content_id_supports_ron_serialization() {
        let original = ContentId::new("classic_1_12_1:warrior.charge").unwrap();

        let serialized = ron::to_string(&original).unwrap();
        let restored: ContentId = ron::from_str(&serialized).unwrap();

        assert_eq!(original, restored);
    }

    #[test]
    fn content_id_rejects_invalid_ron_values() {
        let result = ron::from_str::<ContentId>("\"Classic:Charge\"");

        assert!(result.is_err());
    }
}
