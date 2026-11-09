use super::*;

use crate::content::{
    CombatAttributeDefinition, CombatAttributeValueKind, ContentMetadata, ContentPackManifest,
    DamageSchoolDefinition, DamageSchoolMitigation, EffectDefinition, EffectKind, EffectMagnitude,
    EffectTarget, EffectTrigger,
};

fn content_id(value: &str) -> ContentId {
    ContentId::new(format!("classic_1_12_1:{value}")).unwrap()
}

fn test_database(target: EffectTarget) -> (ContentDatabase, ContentId) {
    let mut database = ContentDatabase::new();

    let pack_id = ContentId::new("pack:classic_1_12_1").unwrap();

    database
        .register_pack(ContentPackManifest {
            id: pack_id.clone(),
            name: "Classic 1.12.1".to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            dependencies: Vec::new(),
        })
        .unwrap();

    let effect_id = content_id("effect.test");

    database
        .register_effect(EffectDefinition {
            metadata: ContentMetadata {
                id: effect_id.clone(),
                pack_id,
                name: "Test Effect".to_string(),
                description: String::new(),
                icon: None,
                tags: Vec::new(),
            },
            trigger: EffectTrigger::OnCast,
            target,
            kind: EffectKind::Interrupt,
        })
        .unwrap();

    (database, effect_id)
}

fn damage_test_database() -> (ContentDatabase, ContentId) {
    let mut database = ContentDatabase::new();

    let pack_id = ContentId::new("pack:classic_1_12_1").unwrap();

    database
        .register_pack(ContentPackManifest {
            id: pack_id.clone(),
            name: "Classic 1.12.1".to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            dependencies: Vec::new(),
        })
        .unwrap();

    let armor_id = content_id("stat.armor");

    database
        .register_combat_attribute(CombatAttributeDefinition {
            metadata: ContentMetadata {
                id: armor_id,
                pack_id: pack_id.clone(),
                name: "Armor".to_string(),
                description: String::new(),
                icon: None,
                tags: Vec::new(),
            },
            value_kind: CombatAttributeValueKind::Rating,
        })
        .unwrap();

    let school_id = content_id("school.physical");

    database
        .register_damage_school(DamageSchoolDefinition {
            metadata: ContentMetadata {
                id: school_id.clone(),
                pack_id: pack_id.clone(),
                name: "Physical".to_string(),
                description: String::new(),
                icon: None,
                tags: Vec::new(),
            },
            mitigation: DamageSchoolMitigation::Armor,
        })
        .unwrap();

    let effect_id = content_id("effect.damage_test");

    database
        .register_effect(EffectDefinition {
            metadata: ContentMetadata {
                id: effect_id.clone(),
                pack_id,
                name: "Damage Test".to_string(),
                description: String::new(),
                icon: None,
                tags: Vec::new(),
            },
            trigger: EffectTrigger::OnCast,
            target: EffectTarget::SelectedTarget,
            kind: EffectKind::Damage {
                amount: EffectMagnitude {
                    base: 40.0,
                    scaling: Vec::new(),
                },
                school: school_id,
            },
        })
        .unwrap();

    database.validate().unwrap();

    (database, effect_id)
}

fn threat_test_database(amount: f32) -> (ContentDatabase, ContentId) {
    let mut database = ContentDatabase::new();

    let pack_id = ContentId::new("pack:classic_1_12_1").unwrap();

    database
        .register_pack(ContentPackManifest {
            id: pack_id.clone(),
            name: "Classic 1.12.1".to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            dependencies: Vec::new(),
        })
        .unwrap();

    let effect_id = content_id("effect.threat_test");

    database
        .register_effect(EffectDefinition {
            metadata: ContentMetadata {
                id: effect_id.clone(),
                pack_id,
                name: "Threat Test".to_string(),
                description: String::new(),
                icon: None,
                tags: Vec::new(),
            },
            trigger: EffectTrigger::OnCast,
            target: EffectTarget::SelectedTarget,
            kind: EffectKind::ModifyThreat {
                amount: EffectMagnitude {
                    base: amount,
                    scaling: Vec::new(),
                },
            },
        })
        .unwrap();

    database.validate().unwrap();

    (database, effect_id)
}

fn source() -> Entity {
    Entity::from_bits(1)
}

fn target() -> Entity {
    Entity::from_bits(2)
}

fn target_two() -> Entity {
    Entity::from_bits(3)
}

fn event_test_effect() -> ResolvedEffect {
    ResolvedEffect {
        effect: content_id("effect.event_test"),
        trigger: EffectTrigger::OnCast,
        source: Some(source()),
        target: target(),
        operation: super::super::ResolvedEffectOperation::Interrupt,
    }
}

mod aura_tests;
mod corruption_tests;
mod custom_effect_tests;
mod displacement_tests;
mod execution_tests;
mod periodic_tests;
mod queue_tests;
