use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::{Display, Formatter};

use super::ContentId;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentPackManifest {
    pub id: ContentId,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub dependencies: Vec<ContentId>,
}

impl ContentPackManifest {
    pub fn validate(&self) -> Result<(), ContentPackValidationError> {
        if self.id.namespace() != "pack" {
            return Err(ContentPackValidationError::InvalidPackIdNamespace);
        }

        if self.name.trim().is_empty() {
            return Err(ContentPackValidationError::EmptyName);
        }

        if self.version.trim().is_empty() {
            return Err(ContentPackValidationError::EmptyVersion);
        }

        let mut dependencies = HashSet::new();

        for dependency in &self.dependencies {
            if dependency.namespace() != "pack" {
                return Err(ContentPackValidationError::InvalidDependencyNamespace);
            }

            if dependency == &self.id {
                return Err(ContentPackValidationError::SelfDependency);
            }

            if !dependencies.insert(dependency) {
                return Err(ContentPackValidationError::DuplicateDependency);
            }
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContentPackValidationError {
    InvalidPackIdNamespace,
    EmptyName,
    EmptyVersion,
    InvalidDependencyNamespace,
    SelfDependency,
    DuplicateDependency,
}

impl Display for ContentPackValidationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InvalidPackIdNamespace => {
                "L'identificativo del pacchetto deve usare il namespace 'pack'"
            }
            Self::EmptyName => "Il nome del pacchetto non può essere vuoto",
            Self::EmptyVersion => "La versione del pacchetto non può essere vuota",
            Self::InvalidDependencyNamespace => "Le dipendenze devono usare il namespace 'pack'",
            Self::SelfDependency => "Un pacchetto non può dipendere da sé stesso",
            Self::DuplicateDependency => "Il pacchetto contiene dipendenze duplicate",
        };

        formatter.write_str(message)
    }
}

impl std::error::Error for ContentPackValidationError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_manifest() -> ContentPackManifest {
        ContentPackManifest {
            id: ContentId::new("pack:classic_1_12_1").unwrap(),
            name: "Classic 1.12.1".to_string(),
            version: "1.0.0".to_string(),
            description: "Contenuti originali Classic".to_string(),
            dependencies: Vec::new(),
        }
    }

    #[test]
    fn valid_manifest_passes_validation() {
        assert!(valid_manifest().validate().is_ok());
    }

    #[test]
    fn manifest_rejects_invalid_pack_namespace() {
        let mut manifest = valid_manifest();
        manifest.id = ContentId::new("classic:content").unwrap();

        assert_eq!(
            manifest.validate(),
            Err(ContentPackValidationError::InvalidPackIdNamespace)
        );
    }

    #[test]
    fn manifest_rejects_empty_name_and_version() {
        let mut manifest = valid_manifest();
        manifest.name = " ".to_string();

        assert_eq!(
            manifest.validate(),
            Err(ContentPackValidationError::EmptyName)
        );

        let mut manifest = valid_manifest();
        manifest.version.clear();

        assert_eq!(
            manifest.validate(),
            Err(ContentPackValidationError::EmptyVersion)
        );
    }

    #[test]
    fn manifest_rejects_invalid_dependencies() {
        let mut manifest = valid_manifest();
        manifest.dependencies = vec![ContentId::new("classic:warrior.charge").unwrap()];

        assert_eq!(
            manifest.validate(),
            Err(ContentPackValidationError::InvalidDependencyNamespace)
        );
    }

    #[test]
    fn manifest_rejects_self_dependency() {
        let mut manifest = valid_manifest();
        manifest.dependencies.push(manifest.id.clone());

        assert_eq!(
            manifest.validate(),
            Err(ContentPackValidationError::SelfDependency)
        );
    }

    #[test]
    fn manifest_rejects_duplicate_dependencies() {
        let mut manifest = valid_manifest();
        let dependency = ContentId::new("pack:shared_base").unwrap();

        manifest.dependencies = vec![dependency.clone(), dependency];

        assert_eq!(
            manifest.validate(),
            Err(ContentPackValidationError::DuplicateDependency)
        );
    }

    #[test]
    fn manifest_supports_ron_serialization() {
        let original = valid_manifest();

        let serialized = ron::to_string(&original).unwrap();
        let restored: ContentPackManifest = ron::from_str(&serialized).unwrap();

        assert_eq!(original, restored);
        assert!(restored.validate().is_ok());
    }
}
