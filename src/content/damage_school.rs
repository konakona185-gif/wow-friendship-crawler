use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

use super::{ContentId, ContentMetadata, ContentMetadataError};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DamageSchoolDefinition {
    pub metadata: ContentMetadata,
    pub mitigation: DamageSchoolMitigation,
}

impl DamageSchoolDefinition {
    pub fn validate(&self) -> Result<(), DamageSchoolValidationError> {
        self.metadata
            .validate()
            .map_err(DamageSchoolValidationError::InvalidMetadata)?;

        self.mitigation.validate()?;

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
    Composite {
        components: Vec<DamageSchoolMitigation>,
    },
}

impl DamageSchoolMitigation {
    fn validate(&self) -> Result<(), DamageSchoolValidationError> {
        let Self::Composite { components } = self else {
            return Ok(());
        };

        if components.len() < 2 {
            return Err(DamageSchoolValidationError::InsufficientCompositeComponents);
        }

        for (index, component) in components.iter().enumerate() {
            if matches!(component, Self::Composite { .. }) {
                return Err(DamageSchoolValidationError::NestedComposite);
            }

            if components[..index].contains(component) {
                return Err(DamageSchoolValidationError::DuplicateCompositeComponent);
            }
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DamageSchoolValidationError {
    InvalidMetadata(ContentMetadataError),
    InsufficientCompositeComponents,
    NestedComposite,
    DuplicateCompositeComponent,
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
            Self::InsufficientCompositeComponents => {
                formatter.write_str("Una scuola composta deve contenere almeno due componenti")
            }
            Self::NestedComposite => formatter.write_str(
                "Una componente di una scuola composta non può essere a sua volta composta",
            ),
            Self::DuplicateCompositeComponent => {
                formatter.write_str("Una scuola composta non può contenere componenti duplicati")
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
                id: ContentId::new("classic_1_12_1:school.physical").unwrap(),
                pack_id: ContentId::new("pack:classic_1_12_1").unwrap(),
                name: "Physical".to_string(),
                description: String::new(),
                icon: None,
                tags: Vec::new(),
            },
            mitigation: DamageSchoolMitigation::Armor,
        }
    }

    fn resistance(stat: &str) -> DamageSchoolMitigation {
        DamageSchoolMitigation::Resistance {
            stat: ContentId::new(stat).unwrap(),
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
            Err(DamageSchoolValidationError::InvalidMetadata(
                ContentMetadataError::EmptyName
            ))
        );
    }

    #[test]
    fn damage_school_supports_ron_serialization() {
        let mut original = valid_school();

        original.mitigation = resistance("classic_1_12_1:stat.fire_resistance");

        let serialized = ron::to_string(&original).unwrap();
        let restored: DamageSchoolDefinition = ron::from_str(&serialized).unwrap();

        assert_eq!(original, restored);
        assert!(restored.validate().is_ok());
    }

    #[test]
    fn composite_damage_school_passes_validation() {
        let mut school = valid_school();

        school.mitigation = DamageSchoolMitigation::Composite {
            components: vec![
                resistance("classic_1_12_1:stat.shadow_resistance"),
                resistance("classic_1_12_1:stat.frost_resistance"),
            ],
        };

        assert!(school.validate().is_ok());
    }

    #[test]
    fn composite_damage_school_supports_ron_serialization() {
        let mut original = valid_school();

        original.mitigation = DamageSchoolMitigation::Composite {
            components: vec![
                resistance("classic_1_12_1:stat.shadow_resistance"),
                resistance("classic_1_12_1:stat.frost_resistance"),
                DamageSchoolMitigation::Armor,
            ],
        };

        let serialized = ron::to_string(&original).unwrap();
        let restored: DamageSchoolDefinition = ron::from_str(&serialized).unwrap();

        assert_eq!(original, restored);
        assert!(restored.validate().is_ok());
    }

    #[test]
    fn composite_damage_school_rejects_fewer_than_two_components() {
        let mut school = valid_school();

        school.mitigation = DamageSchoolMitigation::Composite {
            components: vec![resistance("classic_1_12_1:stat.fire_resistance")],
        };

        assert_eq!(
            school.validate(),
            Err(DamageSchoolValidationError::InsufficientCompositeComponents)
        );
    }

    #[test]
    fn composite_damage_school_rejects_nested_components() {
        let mut school = valid_school();

        school.mitigation = DamageSchoolMitigation::Composite {
            components: vec![
                DamageSchoolMitigation::Armor,
                DamageSchoolMitigation::Composite {
                    components: vec![
                        resistance("classic_1_12_1:stat.shadow_resistance"),
                        resistance("classic_1_12_1:stat.frost_resistance"),
                    ],
                },
            ],
        };

        assert_eq!(
            school.validate(),
            Err(DamageSchoolValidationError::NestedComposite)
        );
    }

    #[test]
    fn composite_damage_school_rejects_duplicate_components() {
        let mut school = valid_school();

        let frost = resistance("classic_1_12_1:stat.frost_resistance");

        school.mitigation = DamageSchoolMitigation::Composite {
            components: vec![frost.clone(), frost],
        };

        assert_eq!(
            school.validate(),
            Err(DamageSchoolValidationError::DuplicateCompositeComponent)
        );
    }
}
