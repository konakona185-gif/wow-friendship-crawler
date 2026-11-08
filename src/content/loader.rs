use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

use super::{
    AbilityDefinition, AuraDefinition, CombatAttributeDefinition, ContentDatabase,
    ContentDatabaseError, ContentId, ContentMetadata, ContentPackManifest, CreatureDefinition,
    DamageSchoolDefinition, EffectDefinition, ResourceDefinition, TalentDefinition,
    TalentTreeDefinition,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContentPackData {
    pub manifest: ContentPackManifest,
    #[serde(default)]
    pub damage_schools: Vec<DamageSchoolDefinition>,
    #[serde(default)]
    pub combat_attributes: Vec<CombatAttributeDefinition>,
    #[serde(default)]
    pub resources: Vec<ResourceDefinition>,
    #[serde(default)]
    pub effects: Vec<EffectDefinition>,
    #[serde(default)]
    pub auras: Vec<AuraDefinition>,
    #[serde(default)]
    pub creatures: Vec<CreatureDefinition>,
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

        for school in pack.damage_schools {
            validate_content_pack(&school.metadata, &expected_pack)?;

            staged_database
                .register_damage_school(school)
                .map_err(ContentPackLoaderError::Database)?;
        }

        for attribute in pack.combat_attributes {
            validate_content_pack(&attribute.metadata, &expected_pack)?;

            staged_database
                .register_combat_attribute(attribute)
                .map_err(ContentPackLoaderError::Database)?;
        }

        for resource in pack.resources {
            validate_content_pack(&resource.metadata, &expected_pack)?;

            staged_database
                .register_resource(resource)
                .map_err(ContentPackLoaderError::Database)?;
        }

        for creature in pack.creatures {
            validate_content_pack(&creature.metadata, &expected_pack)?;

            staged_database
                .register_creature(creature)
                .map_err(ContentPackLoaderError::Database)?;
        }

        for effect in pack.effects {
            validate_content_pack(&effect.metadata, &expected_pack)?;

            staged_database
                .register_effect(effect)
                .map_err(ContentPackLoaderError::Database)?;
        }

        for aura in pack.auras {
            validate_content_pack(&aura.metadata, &expected_pack)?;

            staged_database
                .register_aura(aura)
                .map_err(ContentPackLoaderError::Database)?;
        }

        for ability in pack.abilities {
            validate_content_pack(&ability.metadata, &expected_pack)?;

            staged_database
                .register_ability(ability)
                .map_err(ContentPackLoaderError::Database)?;
        }

        for tree in pack.talent_trees {
            validate_content_pack(&tree.metadata, &expected_pack)?;

            staged_database
                .register_talent_tree(tree)
                .map_err(ContentPackLoaderError::Database)?;
        }

        for talent in pack.talents {
            validate_content_pack(&talent.metadata, &expected_pack)?;

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
mod tests;
