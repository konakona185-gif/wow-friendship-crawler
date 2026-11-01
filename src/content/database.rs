mod lookup;
mod registration;
mod validation;

#[cfg(test)]
mod tests;

use std::collections::HashMap;

use super::{
    AbilityDefinition,
    ContentId,
    ContentMetadata,
    ContentPackManifest,
    ContentPackRegistry,
    EffectDefinition,
    TalentDefinition,
    TalentTreeDefinition,
};

pub use validation::ContentDatabaseError;

#[derive(Clone, Debug, Default)]
pub struct ContentDatabase {
    packs: ContentPackRegistry,
    effects: HashMap<ContentId, EffectDefinition>,
    abilities: HashMap<ContentId, AbilityDefinition>,
    talent_trees: HashMap<ContentId, TalentTreeDefinition>,
    talents: HashMap<ContentId, TalentDefinition>,
}

impl ContentDatabase {
    pub fn new() -> Self {
        Self::default()
    }
}
