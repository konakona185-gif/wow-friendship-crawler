use super::*;

#[test]
fn loader_registers_a_valid_pack() {
    let mut database = ContentDatabase::new();

    ContentPackLoader::load(&mut database, empty_pack()).unwrap();

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

    ContentPackLoader::load(&mut database, pack).unwrap();

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

    ContentPackLoader::load(&mut database, pack).unwrap();

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

    ContentPackLoader::load(&mut database, pack).unwrap();

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

    ContentPackLoader::load(&mut database, pack).unwrap();

    assert_eq!(database.aura_count(), 1);
    assert!(database.aura(&aura_id).is_some());
}

#[test]
fn loader_registers_creatures() {
    let mut database = ContentDatabase::new();
    let mut pack = empty_pack();
    let creature = test_creature();
    let creature_id = creature.metadata.id.clone();

    pack.creatures.push(creature);

    ContentPackLoader::load(&mut database, pack).unwrap();

    assert_eq!(database.creature_count(), 1);
    assert!(database.creature(&creature_id).is_some());
}
