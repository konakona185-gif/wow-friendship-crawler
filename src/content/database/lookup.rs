use super::{
    AbilityDefinition,
    ContentDatabase,
    ContentDatabaseError,
    ContentId,
    ContentPackManifest,
    ContentPackRegistry,
    EffectDefinition,
    TalentDefinition,
    TalentTreeDefinition,
};

impl ContentDatabase {
    pub fn effect(&self, id: &ContentId) -> Option<&EffectDefinition> {
        self.effects.get(id)
    }

    pub fn ability(&self, id: &ContentId) -> Option<&AbilityDefinition> {
        self.abilities.get(id)
    }

    pub fn talent_tree(
        &self,
        id: &ContentId,
    ) -> Option<&TalentTreeDefinition> {
        self.talent_trees.get(id)
    }

    pub fn talent(&self, id: &ContentId) -> Option<&TalentDefinition> {
        self.talents.get(id)
    }

    pub fn packs(&self) -> &ContentPackRegistry {
        &self.packs
    }

    pub fn effect_count(&self) -> usize {
        self.effects.len()
    }

    pub fn ability_count(&self) -> usize {
        self.abilities.len()
    }

    pub fn talent_tree_count(&self) -> usize {
        self.talent_trees.len()
    }

    pub fn talent_count(&self) -> usize {
        self.talents.len()
    }

    pub fn content_count(&self) -> usize {
        self.effect_count()
            + self.ability_count()
            + self.talent_tree_count()
            + self.talent_count()
    }

    pub fn pack_load_order(
        &self,
    ) -> Result<Vec<&ContentPackManifest>, ContentDatabaseError> {
        self.packs
            .load_order()
            .map_err(ContentDatabaseError::InvalidPackRegistry)
    }
}
