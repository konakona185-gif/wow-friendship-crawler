use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

use super::{
    ContentId,
    ContentMetadata,
    ContentMetadataError,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DamageSchoolDefinition {
    pub metadata: ContentMetadata,
    pub mitigation: DamageSchoolMitigation,
}

impl DamageSchoolDefinition {
    pub fn validate(
        &self,
    ) -> Result<(), DamageSchoolValidationError> {
        self.metadata
            .validate()
            .map_err(DamageSchoolValidationError::InvalidMetadata)?;

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "parameters", rename_all = "snake_case")]
pub enum DamageSchoolMitigation {
    Unmitigated,
    Armor,
    Resistance {
        stat: ContentId,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DamageSchoolValidationError {
    InvalidMetadata(ContentMetadataError),
}

impl Display for DamageSchoolValidationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidMetadata(error) => {
                write!(
                    formatter,
                    "Metadati della scuola di danno non validi: {error}"
                )
            }
        }
    }
}

impl std::error::Error for DamageSchoolValidationError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_school() -> DamageSchoolDefinition {
        DamageSchoolDefinition {
            metadata: ContentMetadata {
                id: ContentId::new(
                    "classic_1_12_1:school.physical",
                )
                .unwrap(),
                pack_id: ContentId::new(
                    "pack:classic_1_12_1",
                )
                .unwrap(),
                name: "Physical".to_string(),
                description: String::new(),
                icon: None,
                tags: Vec::new(),
            },
            mitigation: DamageSchoolMitigation::Armor,
        }
    }

    #[test]
    fn valid_damage_school_passes_validation() {
        assert!(valid_school().validate().is_ok());
    }

    #[test]
    fn damage_school_rejects_invalid_metadata() {
        let mut school = valid_school();
        school.metadata.name.clear();

        assert_eq!(
            school.validate(),
            Err(
                DamageSchoolValidationError::InvalidMetadata(
                    ContentMetadataError::EmptyName
                )
            )
        );
    }

    #[test]
    fn damage_school_supports_ron_serialization() {
        let mut original = valid_school();

        original.mitigation = DamageSchoolMitigation::Resistance {
            stat: ContentId::new(
                "classic_1_12_1:stat.fire_resistance",
            )
            .unwrap(),
        };

        let serialized = ron::to_string(&original).unwrap();
        let restored: DamageSchoolDefinition =
            ron::from_str(&serialized).unwrap();

        assert_eq!(original, restored);
        assert!(restored.validate().is_ok());
    }
}
