use super::*;

#[test]
fn pack_data_supports_ron_serialization() {
    let mut original = empty_pack();
    original.damage_schools.push(physical_damage_school());
    original.combat_attributes.push(armor_attribute());
    original.resources.push(mana_resource());
    original.auras.push(test_aura());
    original.creatures.push(test_creature());

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
    assert!(restored.creatures.is_empty());
}
