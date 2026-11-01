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

impl ContentDatabase {
    pub fn register_pack(
        &mut self,
        manifest: ContentPackManifest,
    ) -> Result<(), ContentDatabaseError> {
        self.packs
            .register(manifest)
            .map_err(ContentDatabaseError::InvalidPackRegistry)
    }

    pub fn register_effect(
        &mut self,
        effect: EffectDefinition,
    ) -> Result<(), ContentDatabaseError> {
        effect
            .validate()
            .map_err(ContentDatabaseError::InvalidEffect)?;

        self.validate_new_content(&effect.metadata)?;

        let id = effect.metadata.id.clone();
        self.effects.insert(id, effect);

        Ok(())
    }

    pub fn register_ability(
        &mut self,
        ability: AbilityDefinition,
    ) -> Result<(), ContentDatabaseError> {
        ability
            .validate()
            .map_err(ContentDatabaseError::InvalidAbility)?;

        self.validate_new_content(&ability.metadata)?;

        let id = ability.metadata.id.clone();
        self.abilities.insert(id, ability);

        Ok(())
    }

    pub fn register_talent_tree(
        &mut self,
        tree: TalentTreeDefinition,
    ) -> Result<(), ContentDatabaseError> {
        tree.validate()
            .map_err(ContentDatabaseError::InvalidTalentTree)?;

        self.validate_new_content(&tree.metadata)?;

        let id = tree.metadata.id.clone();
        self.talent_trees.insert(id, tree);

        Ok(())
    }

    pub fn register_talent(
        &mut self,
        talent: TalentDefinition,
    ) -> Result<(), ContentDatabaseError> {
        talent
            .validate()
            .map_err(ContentDatabaseError::InvalidTalent)?;

        self.validate_new_content(&talent.metadata)?;

        let id = talent.metadata.id.clone();
        self.talents.insert(id, talent);

        Ok(())
    }

    fn validate_new_content(
        &self,
        metadata: &ContentMetadata,
    ) -> Result<(), ContentDatabaseError> {
        if self.packs.get(&metadata.pack_id).is_none() {
            return Err(ContentDatabaseError::UnknownPack(
                metadata.pack_id.clone(),
            ));
        }

        let id = &metadata.id;

        if self.effects.contains_key(id)
            || self.abilities.contains_key(id)
            || self.talent_trees.contains_key(id)
            || self.talents.contains_key(id)
        {
            return Err(ContentDatabaseError::DuplicateContent(id.clone()));
        }

        Ok(())
    }
}
