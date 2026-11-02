use super::{ContentDatabase, ContentDatabaseError};
use crate::content::{
    AbilityActivation,
    AbilityDefinition,
    AbilityRange,
    AbilityTargeting,
    ContentId,
    ContentMetadata,
    ContentPackManifest,
    DamageSchoolDefinition,
    DamageSchoolMitigation,
    EffectDefinition,
    EffectKind,
    EffectMagnitude,
    EffectTarget,
    EffectTrigger,
    TalentDefinition,
    TalentPosition,
    TalentPrerequisite,
    TalentRankDefinition,
    TalentTreeDefinition,
};

fn manifest(id: &str, dependencies: &[&str]) -> ContentPackManifest {
    ContentPackManifest {
        id: ContentId::new(id).unwrap(),
        name: id.to_string(),
        version: "1.0.0".to_string(),
        description: String::new(),
        dependencies: dependencies
            .iter()
            .map(|dependency| ContentId::new(*dependency).unwrap())
            .collect(),
    }
}

fn metadata(id: &str, name: &str) -> ContentMetadata {
    ContentMetadata {
        id: ContentId::new(id).unwrap(),
        pack_id: ContentId::new("pack:classic").unwrap(),
        name: name.to_string(),
        description: String::new(),
        icon: None,
        tags: Vec::new(),
    }
}

fn physical_damage_school() -> DamageSchoolDefinition {
    DamageSchoolDefinition {
        metadata: metadata("classic:school.physical", "Physical"),
        mitigation: DamageSchoolMitigation::Armor,
    }
}

fn damage_effect() -> EffectDefinition {
    EffectDefinition {
        metadata: metadata("classic:effect.strike", "Strike"),
        trigger: EffectTrigger::OnCast,
        target: EffectTarget::SelectedTarget,
        kind: EffectKind::Damage {
            amount: EffectMagnitude {
                base: 100.0,
                scaling: Vec::new(),
            },
            school: ContentId::new("classic:school.physical").unwrap(),
        },
    }
}

fn ability() -> AbilityDefinition {
    AbilityDefinition {
        metadata: metadata("classic:ability.strike", "Strike"),
        activation: AbilityActivation::Instant,
        targeting: AbilityTargeting::HostileUnit,
        range: AbilityRange {
            min_range: 0.0,
            max_range: 5.0,
        },
        cooldown_seconds: 3.0,
        costs: Vec::new(),
        effects: vec![
            ContentId::new("classic:effect.strike").unwrap(),
        ],
        class_restrictions: vec![
            ContentId::new("classic:class.warrior").unwrap(),
        ],
        prerequisites: Vec::new(),
    }
}

fn talent_tree(talent_ids: Vec<ContentId>) -> TalentTreeDefinition {
    TalentTreeDefinition {
        metadata: metadata("classic:tree.arms", "Arms"),
        class: ContentId::new("classic:class.warrior").unwrap(),
        display_order: 0,
        talents: talent_ids,
    }
}

fn talent(
    id: &str,
    prerequisites: Vec<TalentPrerequisite>,
) -> TalentDefinition {
    TalentDefinition {
        metadata: metadata(id, id),
        tree: ContentId::new("classic:tree.arms").unwrap(),
        position: TalentPosition {
            row: 0,
            column: 0,
        },
        required_tree_points: 0,
        points_per_rank: 1,
        prerequisites,
        ranks: vec![
            TalentRankDefinition {
                rank: 1,
                effects: vec![
                    ContentId::new("classic:effect.strike").unwrap(),
                ],
                unlocks: vec![
                    ContentId::new("classic:ability.strike").unwrap(),
                ],
            },
        ],
    }
}

fn database_with_classic_pack() -> ContentDatabase {
    let mut database = ContentDatabase::new();
    database.register_pack(manifest("pack:classic", &[])).unwrap();
    database
}

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
    ability.metadata.id =
        ContentId::new("classic:effect.strike").unwrap();

    database.register_effect(damage_effect()).unwrap();

    assert_eq!(
        database.register_ability(ability),
        Err(ContentDatabaseError::DuplicateContent(
            ContentId::new("classic:effect.strike").unwrap()
        ))
    );
}

#[test]
fn database_rejects_damage_effects_with_unknown_schools() {
    let mut database = database_with_classic_pack();
    database.register_effect(damage_effect()).unwrap();

    assert_eq!(
        database.validate(),
        Err(ContentDatabaseError::MissingReference {
            owner: ContentId::new("classic:effect.strike").unwrap(),
            reference: ContentId::new("classic:school.physical").unwrap(),
        })
    );
}

#[test]
fn database_validates_ability_effect_references() {
    let mut database = database_with_classic_pack();
    database.register_ability(ability()).unwrap();

    assert_eq!(
        database.validate(),
        Err(ContentDatabaseError::MissingReference {
            owner: ContentId::new("classic:ability.strike").unwrap(),
            reference: ContentId::new("classic:effect.strike").unwrap(),
        })
    );

    database
        .register_damage_school(physical_damage_school())
        .unwrap();
    database.register_effect(damage_effect()).unwrap();

    assert!(database.validate().is_ok());
}

#[test]
fn database_validates_complete_talent_definitions() {
    let mut database = database_with_classic_pack();

    database
        .register_damage_school(physical_damage_school())
        .unwrap();
    database.register_effect(damage_effect()).unwrap();
    database.register_ability(ability()).unwrap();

    let talent_id = ContentId::new("classic:talent.test").unwrap();

    database
        .register_talent_tree(talent_tree(vec![talent_id.clone()]))
        .unwrap();

    database
        .register_talent(talent(
            "classic:talent.test",
            Vec::new(),
        ))
        .unwrap();

    assert!(database.validate().is_ok());
    assert_eq!(database.talent_tree_count(), 1);
    assert_eq!(database.talent_count(), 1);
    assert_eq!(database.ability_count(), 1);
}

#[test]
fn database_rejects_talent_tree_mismatches() {
    let mut database = database_with_classic_pack();

    let talent_id = ContentId::new("classic:talent.test").unwrap();

    database
        .register_talent_tree(talent_tree(vec![talent_id.clone()]))
        .unwrap();

    let mut mismatched_talent = talent(
        "classic:talent.test",
        Vec::new(),
    );
    mismatched_talent.tree =
        ContentId::new("classic:tree.fury").unwrap();

    database
        .register_talent(mismatched_talent)
        .unwrap();

    assert_eq!(
        database.validate(),
        Err(ContentDatabaseError::TalentTreeMismatch {
            talent: talent_id,
            tree: ContentId::new("classic:tree.arms").unwrap(),
        })
    );
}

#[test]
fn database_rejects_talent_prerequisite_cycles() {
    let mut database = database_with_classic_pack();

    database
        .register_damage_school(physical_damage_school())
        .unwrap();
    database.register_effect(damage_effect()).unwrap();
    database.register_ability(ability()).unwrap();

    let first_id = ContentId::new("classic:talent.first").unwrap();
    let second_id = ContentId::new("classic:talent.second").unwrap();

    database
        .register_talent_tree(talent_tree(vec![
            first_id.clone(),
            second_id.clone(),
        ]))
        .unwrap();

    database
        .register_talent(talent(
            "classic:talent.first",
            vec![TalentPrerequisite {
                talent: second_id.clone(),
                required_rank: 1,
            }],
        ))
        .unwrap();

    database
        .register_talent(talent(
            "classic:talent.second",
            vec![TalentPrerequisite {
                talent: first_id,
                required_rank: 1,
            }],
        ))
        .unwrap();

    assert!(matches!(
        database.validate(),
        Err(ContentDatabaseError::TalentPrerequisiteCycle(_))
    ));
}

#[test]
fn database_resolves_pack_dependencies() {
    let mut database = ContentDatabase::new();

    database
        .register_pack(manifest(
            "pack:expansion",
            &["pack:classic"],
        ))
        .unwrap();

    database
        .register_pack(manifest("pack:classic", &[]))
        .unwrap();

    let order = database.pack_load_order().unwrap();

    let ids: Vec<&str> = order
        .iter()
        .map(|pack| pack.id.as_str())
        .collect();

    assert_eq!(ids, vec!["pack:classic", "pack:expansion"]);
}
