use super::{
    AbilityDefinition, AuraDefinition, CombatAttributeDefinition, ContentDatabase,
    ContentDatabaseError, ContentId, ContentMetadata, ContentPackManifest, CreatureDefinition,
    DamageSchoolDefinition, EffectDefinition, ResourceDefinition, TalentDefinition,
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

    pub fn register_aura(&mut self, aura: AuraDefinition) -> Result<(), ContentDatabaseError> {
        aura.validate().map_err(ContentDatabaseError::InvalidAura)?;

        self.validate_new_content(&aura.metadata)?;

        let id = aura.metadata.id.clone();
        self.auras.insert(id, aura);

        Ok(())
    }

    pub fn register_creature(
        &mut self,
        creature: CreatureDefinition,
    ) -> Result<(), ContentDatabaseError> {
        creature
            .validate()
            .map_err(ContentDatabaseError::InvalidCreature)?;

        self.validate_new_content(&creature.metadata)?;

        let id = creature.metadata.id.clone();
        self.creatures.insert(id, creature);

        Ok(())
    }

    pub fn register_damage_school(
        &mut self,
        school: DamageSchoolDefinition,
    ) -> Result<(), ContentDatabaseError> {
        school
            .validate()
            .map_err(ContentDatabaseError::InvalidDamageSchool)?;

        self.validate_new_content(&school.metadata)?;

        let id = school.metadata.id.clone();
        self.damage_schools.insert(id, school);

        Ok(())
    }

    pub fn register_combat_attribute(
        &mut self,
        attribute: CombatAttributeDefinition,
    ) -> Result<(), ContentDatabaseError> {
        attribute
            .validate()
            .map_err(ContentDatabaseError::InvalidCombatAttribute)?;

        self.validate_new_content(&attribute.metadata)?;

        let id = attribute.metadata.id.clone();
        self.combat_attributes.insert(id, attribute);

        Ok(())
    }

    pub fn register_resource(
        &mut self,
        resource: ResourceDefinition,
    ) -> Result<(), ContentDatabaseError> {
        resource
            .validate()
            .map_err(ContentDatabaseError::InvalidResource)?;

        self.validate_new_content(&resource.metadata)?;

        let id = resource.metadata.id.clone();
        self.resources.insert(id, resource);

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

    fn validate_new_content(&self, metadata: &ContentMetadata) -> Result<(), ContentDatabaseError> {
        if self.packs.get(&metadata.pack_id).is_none() {
            return Err(ContentDatabaseError::UnknownPack(metadata.pack_id.clone()));
        }

        let id = &metadata.id;

        if self.effects.contains_key(id)
            || self.auras.contains_key(id)
            || self.creatures.contains_key(id)
            || self.damage_schools.contains_key(id)
            || self.combat_attributes.contains_key(id)
            || self.resources.contains_key(id)
            || self.abilities.contains_key(id)
            || self.talent_trees.contains_key(id)
            || self.talents.contains_key(id)
        {
            return Err(ContentDatabaseError::DuplicateContent(id.clone()));
        }

        Ok(())
    }
}
