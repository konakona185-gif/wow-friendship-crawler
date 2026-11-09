mod lookup;
mod registration;
mod validation;

#[cfg(test)]
mod tests;

use std::collections::HashMap;

use bevy::prelude::Resource;

use super::{
    AbilityDefinition, AuraDefinition, CombatAttributeDefinition, ContentId, ContentMetadata,
    ContentPackManifest, ContentPackRegistry, CreatureDefinition, DamageSchoolDefinition,
    EffectDefinition, ResourceDefinition, TalentDefinition, TalentTreeDefinition,
};

pub use validation::ContentDatabaseError;

#[derive(Clone, Debug, Default, Resource)]
pub struct ContentDatabase {
    packs: ContentPackRegistry,
    effects: HashMap<ContentId, EffectDefinition>,
    auras: HashMap<ContentId, AuraDefinition>,
    creatures: HashMap<ContentId, CreatureDefinition>,
    damage_schools: HashMap<ContentId, DamageSchoolDefinition>,
    combat_attributes: HashMap<ContentId, CombatAttributeDefinition>,
    resources: HashMap<ContentId, ResourceDefinition>,
    abilities: HashMap<ContentId, AbilityDefinition>,
    talent_trees: HashMap<ContentId, TalentTreeDefinition>,
    talents: HashMap<ContentId, TalentDefinition>,
}

impl ContentDatabase {
    pub fn new() -> Self {
        Self::default()
    }
}
