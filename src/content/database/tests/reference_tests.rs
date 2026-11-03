use super::{
    ability,
    aura,
    aura_effect,
    combat_attribute,
    damage_effect,
    database_with_classic_pack,
    physical_damage_school,
    resource,
    resource_effect,
};
use super::super::ContentDatabaseError;

use crate::content::{
    ContentId,
    DamageSchoolMitigation,
    EffectKind,
    EffectMagnitude,
    EffectScaling,
    ResourceCost,
};

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
    let unknown_stat =
        ContentId::new("classic:stat.fire_resistance").unwrap();

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
    let unknown_stat =
        ContentId::new("classic:stat.fire_resistance").unwrap();
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
    let unknown_stat =
        ContentId::new("classic:stat.spell_power").unwrap();
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
