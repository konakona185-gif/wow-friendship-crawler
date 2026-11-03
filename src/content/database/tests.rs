mod aura_tests;

use super::{ContentDatabase, ContentDatabaseError};
use crate::content::{
    AbilityActivation,
    AbilityDefinition,
    AbilityRange,
    AbilityTargeting,
    AuraDefinition,
    AuraStackingPolicy,
    AuraValidationError,
    CombatAttributeDefinition,
    CombatAttributeValueKind,
    CombatAttributeValidationError,
    ContentId,
    ContentMetadata,
    ContentMetadataError,
    ContentPackManifest,
    DamageSchoolDefinition,
    DamageSchoolMitigation,
    EffectDefinition,
    EffectKind,
    EffectMagnitude,
    EffectScaling,
    EffectTarget,
    EffectTrigger,
    ResourceCost,
    ResourceDefinition,
    ResourceValidationError,
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

fn combat_attribute() -> CombatAttributeDefinition {
    CombatAttributeDefinition {
        metadata: metadata("classic:stat.armor", "Armor"),
        value_kind: CombatAttributeValueKind::Flat,
    }
}

fn resource() -> ResourceDefinition {
    ResourceDefinition {
        metadata: metadata("classic:resource.mana", "Mana"),
    }
}

fn aura() -> AuraDefinition {
    AuraDefinition {
        metadata: metadata("classic:aura.test", "Test Aura"),
        max_stacks: 5,
        stacking_policy: AuraStackingPolicy::AddStacks,
        periodic_effects: Vec::new(),
    }
}

fn aura_effect(
    id: &str,
    aura_id: &str,
    apply: bool,
) -> EffectDefinition {
    let aura = ContentId::new(aura_id).unwrap();

    let kind = if apply {
        EffectKind::ApplyAura {
            aura,
            stacks: 1,
            duration_seconds: Some(10.0),
        }
    } else {
        EffectKind::RemoveAura {
            aura,
            stacks: None,
        }
    };

    EffectDefinition {
        metadata: metadata(id, id),
        trigger: EffectTrigger::OnCast,
        target: EffectTarget::SelectedTarget,
        kind,
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

fn resource_effect(
    id: &str,
    resource_id: &str,
    gain: bool,
) -> EffectDefinition {
    let resource = ContentId::new(resource_id).unwrap();
    let amount = EffectMagnitude {
        base: 10.0,
        scaling: Vec::new(),
    };

    EffectDefinition {
        metadata: metadata(id, id),
        trigger: EffectTrigger::OnCast,
        target: EffectTarget::Caster,
        kind: if gain {
            EffectKind::GainResource { resource, amount }
        } else {
            EffectKind::SpendResource { resource, amount }
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
            AuraValidationError::InvalidMetadata(
                ContentMetadataError::EmptyName
            )
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
    aura.metadata.id =
        ContentId::new("classic:effect.strike").unwrap();

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
fn database_rejects_effects_with_unknown_aura_references() {
    let unknown_aura =
        ContentId::new("classic:aura.missing").unwrap();

    for (effect_id, apply) in [
        ("classic:effect.apply_aura_test", true),
        ("classic:effect.remove_aura_test", false),
    ] {
        let mut database = database_with_classic_pack();

        database
            .register_effect(aura_effect(
                effect_id,
                unknown_aura.as_str(),
                apply,
            ))
            .unwrap();

        assert_eq!(
            database.validate(),
            Err(ContentDatabaseError::MissingReference {
                owner: ContentId::new(effect_id).unwrap(),
                reference: unknown_aura.clone(),
            })
        );
    }
}

#[test]
fn database_accepts_effects_with_registered_aura_references() {
    let mut database = database_with_classic_pack();
    let aura = aura();
    let aura_id = aura.metadata.id.clone();

    database.register_aura(aura).unwrap();

    database
        .register_effect(aura_effect(
            "classic:effect.apply_aura_test",
            aura_id.as_str(),
            true,
        ))
        .unwrap();

    database
        .register_effect(aura_effect(
            "classic:effect.remove_aura_test",
            aura_id.as_str(),
            false,
        ))
        .unwrap();

    assert!(database.validate().is_ok());
    assert_eq!(database.aura_count(), 1);
    assert_eq!(database.effect_count(), 2);
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
            CombatAttributeValidationError::InvalidMetadata(
                ContentMetadataError::EmptyName
            )
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
            ResourceValidationError::InvalidMetadata(
                ContentMetadataError::EmptyName
            )
        ))
    );

    assert_eq!(database.resource_count(), 0);
}

#[test]
fn database_rejects_duplicate_combat_attribute_ids() {
    let mut database = database_with_classic_pack();
    let mut attribute = combat_attribute();
    attribute.metadata.id =
        ContentId::new("classic:effect.strike").unwrap();

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
    resource.metadata.id =
        ContentId::new("classic:effect.strike").unwrap();

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
fn database_rejects_damage_schools_with_unknown_resistance_attributes() {
    let mut database = database_with_classic_pack();
    let mut school = physical_damage_school();
    let unknown_stat = ContentId::new("classic:stat.fire_resistance").unwrap();

    school.mitigation = DamageSchoolMitigation::Resistance {
        stat: unknown_stat.clone(),
    };

    database.register_damage_school(school).unwrap();

    assert_eq!(
        database.validate(),
        Err(ContentDatabaseError::MissingReference {
            owner: ContentId::new("classic:school.physical").unwrap(),
            reference: unknown_stat,
        })
    );
}

#[test]
fn database_rejects_composite_damage_schools_with_unknown_resistance_attributes() {
    let mut database = database_with_classic_pack();
    let unknown_stat = ContentId::new("classic:stat.fire_resistance").unwrap();
    let mut school = physical_damage_school();

    school.mitigation = DamageSchoolMitigation::Composite {
        components: vec![
            DamageSchoolMitigation::Armor,
            DamageSchoolMitigation::Resistance {
                stat: unknown_stat.clone(),
            },
        ],
    };

    database.register_damage_school(school).unwrap();

    assert_eq!(
        database.validate(),
        Err(ContentDatabaseError::MissingReference {
            owner: ContentId::new("classic:school.physical").unwrap(),
            reference: unknown_stat,
        })
    );
}

#[test]
fn database_accepts_composite_damage_schools_with_registered_resistance_attributes() {
    let mut database = database_with_classic_pack();
    let resistance_id =
        ContentId::new("classic:stat.fire_resistance").unwrap();

    let mut attribute = combat_attribute();
    attribute.metadata.id = resistance_id.clone();
    attribute.metadata.name = "Fire Resistance".to_string();

    let mut school = physical_damage_school();
    school.mitigation = DamageSchoolMitigation::Composite {
        components: vec![
            DamageSchoolMitigation::Armor,
            DamageSchoolMitigation::Resistance {
                stat: resistance_id,
            },
        ],
    };

    database.register_combat_attribute(attribute).unwrap();
    database.register_damage_school(school).unwrap();

    assert!(database.validate().is_ok());
}

#[test]
fn database_rejects_effects_with_unknown_scaling_attributes() {
    let mut database = database_with_classic_pack();
    let unknown_stat = ContentId::new("classic:stat.spell_power").unwrap();
    let mut effect = damage_effect();

    effect.kind = EffectKind::Heal {
        amount: EffectMagnitude {
            base: 100.0,
            scaling: vec![
                EffectScaling {
                    stat: unknown_stat.clone(),
                    coefficient: 1.0,
                },
            ],
        },
    };

    database.register_effect(effect).unwrap();

    assert_eq!(
        database.validate(),
        Err(ContentDatabaseError::MissingReference {
            owner: ContentId::new("classic:effect.strike").unwrap(),
            reference: unknown_stat,
        })
    );
}

#[test]
fn database_rejects_effects_with_unknown_resources() {
    let unknown_resource =
        ContentId::new("classic:resource.energy").unwrap();

    for gain in [true, false] {
        let mut database = database_with_classic_pack();
        let effect = resource_effect(
            "classic:effect.resource",
            unknown_resource.as_str(),
            gain,
        );

        database.register_effect(effect).unwrap();

        assert_eq!(
            database.validate(),
            Err(ContentDatabaseError::MissingReference {
                owner: ContentId::new("classic:effect.resource").unwrap(),
                reference: unknown_resource.clone(),
            })
        );
    }
}

#[test]
fn database_rejects_abilities_with_unknown_resource_costs() {
    let mut database = database_with_classic_pack();
    let unknown_resource =
        ContentId::new("classic:resource.energy").unwrap();

    database
        .register_damage_school(physical_damage_school())
        .unwrap();

    database
        .register_effect(damage_effect())
        .unwrap();

    let mut ability = ability();
    ability.costs = vec![
        ResourceCost {
            resource: unknown_resource.clone(),
            amount: 10.0,
        },
    ];

    database.register_ability(ability).unwrap();

    assert_eq!(
        database.validate(),
        Err(ContentDatabaseError::MissingReference {
            owner: ContentId::new("classic:ability.strike").unwrap(),
            reference: unknown_resource,
        })
    );
}

#[test]
fn database_accepts_registered_combat_attribute_references() {
    let mut database = database_with_classic_pack();
    let attribute = combat_attribute();
    let attribute_id = attribute.metadata.id.clone();
    let mut school = physical_damage_school();

    school.mitigation = DamageSchoolMitigation::Resistance {
        stat: attribute_id.clone(),
    };

    let mut effect = damage_effect();
    effect.kind = EffectKind::Heal {
        amount: EffectMagnitude {
            base: 100.0,
            scaling: vec![
                EffectScaling {
                    stat: attribute_id,
                    coefficient: 1.0,
                },
            ],
        },
    };

    database.register_combat_attribute(attribute).unwrap();
    database.register_damage_school(school).unwrap();
    database.register_effect(effect).unwrap();

    assert!(database.validate().is_ok());
}

#[test]
fn database_accepts_registered_resource_references() {
    let mut database = database_with_classic_pack();
    let resource = resource();
    let resource_id = resource.metadata.id.clone();

    let gain_effect = resource_effect(
        "classic:effect.gain_mana",
        resource_id.as_str(),
        true,
    );
    let spend_effect = resource_effect(
        "classic:effect.spend_mana",
        resource_id.as_str(),
        false,
    );

    let gain_effect_id = gain_effect.metadata.id.clone();
    let spend_effect_id = spend_effect.metadata.id.clone();

    database.register_resource(resource).unwrap();
    database.register_effect(gain_effect).unwrap();
    database.register_effect(spend_effect).unwrap();

    let mut ability = ability();
    ability.effects = vec![gain_effect_id, spend_effect_id];
    ability.costs = vec![
        ResourceCost {
            resource: resource_id,
            amount: 10.0,
        },
    ];

    database.register_ability(ability).unwrap();

    assert!(database.validate().is_ok());
    assert_eq!(database.resource_count(), 1);
    assert_eq!(database.content_count(), 4);
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
