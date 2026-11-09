use super::super::{ContentDatabase, ContentDatabaseError};
use super::{
    ability, aura, combat_attribute, damage_effect, database_with_classic_pack,
    physical_damage_school, resource,
};
use crate::content::{
    AuraValidationError, CombatAttributeValidationError, ContentId, ContentMetadataError,
    ResourceValidationError,
};

#[test]
fn database_registers_and_retrieves_content() {
    let mut database = database_with_classic_pack();
    let effect = damage_effect();
    let id = effect.metadata.id.clone();

    database.register_effect(effect).unwrap();

    assert_eq!(database.effect_count(), 1);
    assert_eq!(database.content_count(), 1);
    assert!(database.effect(&id).is_some());
}

#[test]
fn database_registers_and_retrieves_auras() {
    let mut database = database_with_classic_pack();
    let aura = aura();
    let id = aura.metadata.id.clone();

    database.register_aura(aura).unwrap();

    assert_eq!(database.aura_count(), 1);
    assert_eq!(database.content_count(), 1);
    assert!(database.aura(&id).is_some());
}

#[test]
fn database_rejects_invalid_auras() {
    let mut database = database_with_classic_pack();
    let mut aura = aura();
    aura.metadata.name.clear();

    assert_eq!(
        database.register_aura(aura),
        Err(ContentDatabaseError::InvalidAura(
            AuraValidationError::InvalidMetadata(ContentMetadataError::EmptyName)
        ))
    );

    assert_eq!(database.aura_count(), 0);
    assert_eq!(database.content_count(), 0);
}

#[test]
fn database_rejects_duplicate_ids_between_auras_and_other_content_types() {
    let mut database = database_with_classic_pack();
    database.register_effect(damage_effect()).unwrap();

    let mut aura = aura();
    aura.metadata.id = ContentId::new("classic:effect.strike").unwrap();

    assert_eq!(
        database.register_aura(aura),
        Err(ContentDatabaseError::DuplicateContent(
            ContentId::new("classic:effect.strike").unwrap()
        ))
    );

    assert_eq!(database.aura_count(), 0);
    assert_eq!(database.effect_count(), 1);
}

#[test]
fn database_registers_and_retrieves_damage_schools() {
    let mut database = database_with_classic_pack();
    let school = physical_damage_school();
    let id = school.metadata.id.clone();

    database.register_damage_school(school).unwrap();

    assert_eq!(database.damage_school_count(), 1);
    assert_eq!(database.content_count(), 1);
    assert!(database.damage_school(&id).is_some());
}

#[test]
fn database_registers_and_retrieves_combat_attributes() {
    let mut database = database_with_classic_pack();
    let attribute = combat_attribute();
    let id = attribute.metadata.id.clone();

    database.register_combat_attribute(attribute).unwrap();

    assert_eq!(database.combat_attribute_count(), 1);
    assert_eq!(database.content_count(), 1);
    assert!(database.combat_attribute(&id).is_some());
}

#[test]
fn database_registers_and_retrieves_resources() {
    let mut database = database_with_classic_pack();
    let resource = resource();
    let id = resource.metadata.id.clone();

    database.register_resource(resource).unwrap();

    assert_eq!(database.resource_count(), 1);
    assert_eq!(database.content_count(), 1);
    assert!(database.resource(&id).is_some());
}

#[test]
fn database_rejects_invalid_combat_attributes() {
    let mut database = database_with_classic_pack();
    let mut attribute = combat_attribute();
    attribute.metadata.name.clear();

    assert_eq!(
        database.register_combat_attribute(attribute),
        Err(ContentDatabaseError::InvalidCombatAttribute(
            CombatAttributeValidationError::InvalidMetadata(ContentMetadataError::EmptyName)
        ))
    );

    assert_eq!(database.combat_attribute_count(), 0);
}

#[test]
fn database_rejects_invalid_resources() {
    let mut database = database_with_classic_pack();
    let mut resource = resource();
    resource.metadata.name.clear();

    assert_eq!(
        database.register_resource(resource),
        Err(ContentDatabaseError::InvalidResource(
            ResourceValidationError::InvalidMetadata(ContentMetadataError::EmptyName)
        ))
    );

    assert_eq!(database.resource_count(), 0);
}

#[test]
fn database_rejects_duplicate_combat_attribute_ids() {
    let mut database = database_with_classic_pack();
    let mut attribute = combat_attribute();
    attribute.metadata.id = ContentId::new("classic:effect.strike").unwrap();

    database.register_effect(damage_effect()).unwrap();

    assert_eq!(
        database.register_combat_attribute(attribute),
        Err(ContentDatabaseError::DuplicateContent(
            ContentId::new("classic:effect.strike").unwrap()
        ))
    );

    assert_eq!(database.combat_attribute_count(), 0);
}

#[test]
fn database_rejects_duplicate_resource_ids() {
    let mut database = database_with_classic_pack();
    let mut resource = resource();
    resource.metadata.id = ContentId::new("classic:effect.strike").unwrap();

    database.register_effect(damage_effect()).unwrap();

    assert_eq!(
        database.register_resource(resource),
        Err(ContentDatabaseError::DuplicateContent(
            ContentId::new("classic:effect.strike").unwrap()
        ))
    );

    assert_eq!(database.resource_count(), 0);
}

#[test]
fn database_rejects_content_from_unknown_packs() {
    let mut database = ContentDatabase::new();

    assert_eq!(
        database.register_effect(damage_effect()),
        Err(ContentDatabaseError::UnknownPack(
            ContentId::new("pack:classic").unwrap()
        ))
    );
}

#[test]
fn database_rejects_duplicate_ids_across_content_types() {
    let mut database = database_with_classic_pack();
    let mut ability = ability();
    ability.metadata.id = ContentId::new("classic:effect.strike").unwrap();

    database.register_effect(damage_effect()).unwrap();

    assert_eq!(
        database.register_ability(ability),
        Err(ContentDatabaseError::DuplicateContent(
            ContentId::new("classic:effect.strike").unwrap()
        ))
    );
}
