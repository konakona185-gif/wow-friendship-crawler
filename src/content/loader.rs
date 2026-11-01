use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

use super::{
    AbilityDefinition,
    ContentDatabase,
    ContentDatabaseError,
    ContentId,
    ContentMetadata,
    ContentPackManifest,
    EffectDefinition,
    TalentDefinition,
    TalentTreeDefinition,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContentPackData {
    pub manifest: ContentPackManifest,
    #[serde(default)]
    pub effects: Vec<EffectDefinition>,
    #[serde(default)]
    pub abilities: Vec<AbilityDefinition>,
    #[serde(default)]
    pub talent_trees: Vec<TalentTreeDefinition>,
    #[serde(default)]
    pub talents: Vec<TalentDefinition>,
}

pub struct ContentPackLoader;

impl ContentPackLoader {
    pub fn parse_ron(source: &str) -> Result<ContentPackData, ContentPackLoaderError> {
        ron::from_str(source).map_err(ContentPackLoaderError::InvalidRon)
    }

    pub fn load_ron(
        database: &mut ContentDatabase,
        source: &str,
    ) -> Result<(), ContentPackLoaderError> {
        let pack = Self::parse_ron(source)?;
        Self::load(database, pack)
    }

    pub fn load(
        database: &mut ContentDatabase,
        pack: ContentPackData,
    ) -> Result<(), ContentPackLoaderError> {
        let expected_pack = pack.manifest.id.clone();
        let mut staged_database = database.clone();

        staged_database
            .register_pack(pack.manifest)
            .map_err(ContentPackLoaderError::Database)?;

        for effect in pack.effects {
            validate_content_pack(
                &effect.metadata,
                &expected_pack,
            )?;

            staged_database
                .register_effect(effect)
                .map_err(ContentPackLoaderError::Database)?;
        }

        for ability in pack.abilities {
            validate_content_pack(
                &ability.metadata,
                &expected_pack,
            )?;

            staged_database
                .register_ability(ability)
                .map_err(ContentPackLoaderError::Database)?;
        }

        for tree in pack.talent_trees {
            validate_content_pack(
                &tree.metadata,
                &expected_pack,
            )?;

            staged_database
                .register_talent_tree(tree)
                .map_err(ContentPackLoaderError::Database)?;
        }

        for talent in pack.talents {
            validate_content_pack(
                &talent.metadata,
                &expected_pack,
            )?;

            staged_database
                .register_talent(talent)
                .map_err(ContentPackLoaderError::Database)?;
        }

        staged_database
            .validate()
            .map_err(ContentPackLoaderError::Database)?;

        *database = staged_database;

        Ok(())
    }
}

fn validate_content_pack(
    metadata: &ContentMetadata,
    expected_pack: &ContentId,
) -> Result<(), ContentPackLoaderError> {
    if metadata.pack_id != *expected_pack {
        return Err(ContentPackLoaderError::ContentFromDifferentPack {
            content: metadata.id.clone(),
            expected_pack: expected_pack.clone(),
            actual_pack: metadata.pack_id.clone(),
        });
    }

    Ok(())
}

#[derive(Debug)]
pub enum ContentPackLoaderError {
    InvalidRon(ron::error::SpannedError),
    Database(ContentDatabaseError),
    ContentFromDifferentPack {
        content: ContentId,
        expected_pack: ContentId,
        actual_pack: ContentId,
    },
}

impl Display for ContentPackLoaderError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRon(error) => {
                write!(formatter, "File RON non valido: {error}")
            }
            Self::Database(error) => {
                write!(formatter, "Errore durante il caricamento: {error}")
            }
            Self::ContentFromDifferentPack {
                content,
                expected_pack,
                actual_pack,
            } => {
                write!(
                    formatter,
                    "Il contenuto {} appartiene al pacchetto {}, ma è stato inserito nel pacchetto {}",
                    content.as_str(),
                    actual_pack.as_str(),
                    expected_pack.as_str()
                )
            }
        }
    }
}

impl std::error::Error for ContentPackLoaderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidRon(error) => Some(error),
            Self::Database(error) => Some(error),
            Self::ContentFromDifferentPack { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_pack() -> ContentPackData {
        ContentPackData {
            manifest: ContentPackManifest {
                id: ContentId::new("pack:classic").unwrap(),
                name: "Classic".to_string(),
                version: "1.0.0".to_string(),
                description: String::new(),
                dependencies: Vec::new(),
            },
            effects: Vec::new(),
            abilities: Vec::new(),
            talent_trees: Vec::new(),
            talents: Vec::new(),
        }
    }

    #[test]
    fn pack_data_supports_ron_serialization() {
        let original = empty_pack();

        let serialized = ron::to_string(&original).unwrap();
        let restored = ContentPackLoader::parse_ron(&serialized).unwrap();

        assert_eq!(original, restored);
    }

    #[test]
    fn loader_registers_a_valid_pack() {
        let mut database = ContentDatabase::new();

        ContentPackLoader::load(
            &mut database,
            empty_pack(),
        )
        .unwrap();

        assert_eq!(database.packs().len(), 1);
        assert_eq!(database.pack_load_order().unwrap().len(), 1);
    }

    #[test]
    fn loader_rejects_content_from_another_pack_transactionally() {
        let mut database = ContentDatabase::new();
        let mut pack = empty_pack();

        pack.effects.push(EffectDefinition {
            metadata: ContentMetadata {
                id: ContentId::new("other:effect.test").unwrap(),
                pack_id: ContentId::new("pack:other").unwrap(),
                name: "Test".to_string(),
                description: String::new(),
                icon: None,
                tags: Vec::new(),
            },
            trigger: super::super::EffectTrigger::OnCast,
            target: super::super::EffectTarget::SelectedTarget,
            kind: super::super::EffectKind::Damage {
                amount: super::super::EffectMagnitude {
                    base: 10.0,
                    scaling: Vec::new(),
                },
                school: ContentId::new("classic:school.physical").unwrap(),
            },
        });

        let result = ContentPackLoader::load(
            &mut database,
            pack,
        );

        assert!(matches!(
            result,
            Err(ContentPackLoaderError::ContentFromDifferentPack { .. })
        ));

        assert!(database.packs().is_empty());
    }

    #[test]
    fn loader_rejects_invalid_ron() {
        assert!(ContentPackLoader::parse_ron("invalid ron").is_err());
    }
}
