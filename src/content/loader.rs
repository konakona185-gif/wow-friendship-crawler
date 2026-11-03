use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

use super::{
    AbilityDefinition,
    AuraDefinition,
    CombatAttributeDefinition,
    ContentDatabase,
    ContentDatabaseError,
    ContentId,
    ContentMetadata,
    ContentPackManifest,
    DamageSchoolDefinition,
    EffectDefinition,
    ResourceDefinition,
    TalentDefinition,
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
            validate_content_pack(
                &school.metadata,
                &expected_pack,
            )?;

            staged_database
                .register_damage_school(school)
                .map_err(ContentPackLoaderError::Database)?;
        }

        for attribute in pack.combat_attributes {
            validate_content_pack(
                &attribute.metadata,
                &expected_pack,
            )?;

            staged_database
                .register_combat_attribute(attribute)
                .map_err(ContentPackLoaderError::Database)?;
        }

        for resource in pack.resources {
            validate_content_pack(
                &resource.metadata,
                &expected_pack,
            )?;

            staged_database
                .register_resource(resource)
                .map_err(ContentPackLoaderError::Database)?;
        }

        for effect in pack.effects {
            validate_content_pack(
                &effect.metadata,
                &expected_pack,
            )?;

            staged_database
                .register_effect(effect)
                .map_err(ContentPackLoaderError::Database)?;
        }

        for aura in pack.auras {
            validate_content_pack(
                &aura.metadata,
                &expected_pack,
            )?;

            staged_database
                .register_aura(aura)
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
    use crate::content::{
        AuraStackingPolicy,
        CombatAttributeValueKind,
    };
    use serde::Serialize;

    #[derive(Serialize)]
    struct LegacyContentPackData {
        manifest: ContentPackManifest,
        damage_schools: Vec<DamageSchoolDefinition>,
        combat_attributes: Vec<CombatAttributeDefinition>,
        effects: Vec<EffectDefinition>,
        abilities: Vec<AbilityDefinition>,
        talent_trees: Vec<TalentTreeDefinition>,
        talents: Vec<TalentDefinition>,
    }

    fn empty_pack() -> ContentPackData {
        ContentPackData {
            manifest: ContentPackManifest {
                id: ContentId::new("pack:classic").unwrap(),
                name: "Classic".to_string(),
                version: "1.0.0".to_string(),
                description: String::new(),
                dependencies: Vec::new(),
            },
            damage_schools: Vec::new(),
            combat_attributes: Vec::new(),
            resources: Vec::new(),
            effects: Vec::new(),
            auras: Vec::new(),
            abilities: Vec::new(),
            talent_trees: Vec::new(),
            talents: Vec::new(),
        }
    }

    fn physical_damage_school() -> DamageSchoolDefinition {
        DamageSchoolDefinition {
            metadata: ContentMetadata {
                id: ContentId::new("classic:school.physical").unwrap(),
                pack_id: ContentId::new("pack:classic").unwrap(),
                name: "Physical".to_string(),
                description: String::new(),
                icon: None,
                tags: Vec::new(),
            },
            mitigation: super::super::DamageSchoolMitigation::Armor,
        }
    }

    fn armor_attribute() -> CombatAttributeDefinition {
        CombatAttributeDefinition {
            metadata: ContentMetadata {
                id: ContentId::new("classic:stat.armor").unwrap(),
                pack_id: ContentId::new("pack:classic").unwrap(),
                name: "Armor".to_string(),
                description: String::new(),
                icon: None,
                tags: Vec::new(),
            },
            value_kind: CombatAttributeValueKind::Flat,
        }
    }

    fn mana_resource() -> ResourceDefinition {
        ResourceDefinition {
            metadata: ContentMetadata {
                id: ContentId::new("classic:resource.mana").unwrap(),
                pack_id: ContentId::new("pack:classic").unwrap(),
                name: "Mana".to_string(),
                description: String::new(),
                icon: None,
                tags: Vec::new(),
            },
        }
    }

    fn test_aura() -> AuraDefinition {
        AuraDefinition {
            metadata: ContentMetadata {
                id: ContentId::new("classic:aura.test").unwrap(),
                pack_id: ContentId::new("pack:classic").unwrap(),
                name: "Test Aura".to_string(),
                description: String::new(),
                icon: None,
                tags: Vec::new(),
            },
            max_stacks: 5,
            stacking_policy: AuraStackingPolicy::AddStacks,
            periodic_effects: Vec::new(),
        }
    }

    #[test]
    fn pack_data_supports_ron_serialization() {
        let mut original = empty_pack();
        original.damage_schools.push(physical_damage_school());
        original.combat_attributes.push(armor_attribute());
        original.resources.push(mana_resource());
        original.auras.push(test_aura());

        let serialized = ron::to_string(&original).unwrap();
        let restored = ContentPackLoader::parse_ron(&serialized).unwrap();

        assert_eq!(original, restored);
    }

    #[test]
    fn pack_data_loads_legacy_ron_without_resources_or_auras() {
        let pack = empty_pack();

        let legacy = LegacyContentPackData {
            manifest: pack.manifest,
            damage_schools: pack.damage_schools,
            combat_attributes: pack.combat_attributes,
            effects: pack.effects,
            abilities: pack.abilities,
            talent_trees: pack.talent_trees,
            talents: pack.talents,
        };

        let serialized = ron::to_string(&legacy).unwrap();
        let restored = ContentPackLoader::parse_ron(&serialized).unwrap();

        assert!(restored.resources.is_empty());
        assert!(restored.auras.is_empty());
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
    fn loader_registers_damage_schools() {
        let mut database = ContentDatabase::new();
        let mut pack = empty_pack();
        let school = physical_damage_school();
        let school_id = school.metadata.id.clone();

        pack.damage_schools.push(school);

        ContentPackLoader::load(
            &mut database,
            pack,
        )
        .unwrap();

        assert_eq!(database.damage_school_count(), 1);
        assert!(database.damage_school(&school_id).is_some());
    }

    #[test]
    fn loader_registers_combat_attributes() {
        let mut database = ContentDatabase::new();
        let mut pack = empty_pack();
        let attribute = armor_attribute();
        let attribute_id = attribute.metadata.id.clone();

        pack.combat_attributes.push(attribute);

        ContentPackLoader::load(
            &mut database,
            pack,
        )
        .unwrap();

        assert_eq!(database.combat_attribute_count(), 1);
        assert!(database.combat_attribute(&attribute_id).is_some());
    }

    #[test]
    fn loader_registers_resources() {
        let mut database = ContentDatabase::new();
        let mut pack = empty_pack();
        let resource = mana_resource();
        let resource_id = resource.metadata.id.clone();

        pack.resources.push(resource);

        ContentPackLoader::load(
            &mut database,
            pack,
        )
        .unwrap();

        assert_eq!(database.resource_count(), 1);
        assert!(database.resource(&resource_id).is_some());
    }

    #[test]
    fn loader_registers_auras() {
        let mut database = ContentDatabase::new();
        let mut pack = empty_pack();
        let aura = test_aura();
        let aura_id = aura.metadata.id.clone();

        pack.auras.push(aura);

        ContentPackLoader::load(
            &mut database,
            pack,
        )
        .unwrap();

        assert_eq!(database.aura_count(), 1);
        assert!(database.aura(&aura_id).is_some());
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
