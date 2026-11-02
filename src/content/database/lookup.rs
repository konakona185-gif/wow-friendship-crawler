use super::{
    AbilityDefinition,
    CombatAttributeDefinition,
    ContentDatabase,
    ContentDatabaseError,
    ContentId,
    ContentPackManifest,
    ContentPackRegistry,
    DamageSchoolDefinition,
    EffectDefinition,
    TalentDefinition,
    TalentTreeDefinition,
};

impl ContentDatabase {
    pub fn effect(&self, id: &ContentId) -> Option<&EffectDefinition> {
        self.effects.get(id)
    }

    pub fn damage_school(
        &self,
        id: &ContentId,
    ) -> Option<&DamageSchoolDefinition> {
        self.damage_schools.get(id)
    }

    pub fn combat_attribute(
        &self,
        id: &ContentId,
    ) -> Option<&CombatAttributeDefinition> {
        self.combat_attributes.get(id)
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

    pub fn damage_school_count(&self) -> usize {
        self.damage_schools.len()
    }

    pub fn combat_attribute_count(&self) -> usize {
        self.combat_attributes.len()
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
            + self.damage_school_count()
            + self.combat_attribute_count()
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
