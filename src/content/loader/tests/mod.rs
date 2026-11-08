use super::*;
use crate::content::{
    AuraRefreshPolicy,
    AuraStackingPolicy,
    AuraStackingScope,
    CombatAttributeValueKind,
};
use serde::Serialize;

#[derive(Serialize)]
struct LegacyContentPackData {
    manifest: ContentPackManifest,
    damage_schools: Vec<DamageSchoolDefinition>,
    combat_attributes: Vec<CombatAttributeDefinition>,
    effects: Vec<EffectDefinition>,
    abilities: Vec<AbilityDefinition>,
    talent_trees: Vec<TalentTreeDefinition>,
    talents: Vec<TalentDefinition>,
}

fn empty_pack() -> ContentPackData {
    ContentPackData {
        manifest: ContentPackManifest {
            id: ContentId::new("pack:classic").unwrap(),
            name: "Classic".to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            dependencies: Vec::new(),
        },
        damage_schools: Vec::new(),
        combat_attributes: Vec::new(),
        resources: Vec::new(),
        effects: Vec::new(),
        auras: Vec::new(),
        creatures: Vec::new(),
        abilities: Vec::new(),
        talent_trees: Vec::new(),
        talents: Vec::new(),
    }
}

fn physical_damage_school() -> DamageSchoolDefinition {
    DamageSchoolDefinition {
        metadata: ContentMetadata {
            id: ContentId::new("classic:school.physical").unwrap(),
            pack_id: ContentId::new("pack:classic").unwrap(),
            name: "Physical".to_string(),
            description: String::new(),
            icon: None,
            tags: Vec::new(),
        },
        mitigation: super::super::DamageSchoolMitigation::Armor,
    }
}

fn armor_attribute() -> CombatAttributeDefinition {
    CombatAttributeDefinition {
        metadata: ContentMetadata {
            id: ContentId::new("classic:stat.armor").unwrap(),
            pack_id: ContentId::new("pack:classic").unwrap(),
            name: "Armor".to_string(),
            description: String::new(),
            icon: None,
            tags: Vec::new(),
        },
        value_kind: CombatAttributeValueKind::Flat,
    }
}

fn mana_resource() -> ResourceDefinition {
    ResourceDefinition {
        metadata: ContentMetadata {
            id: ContentId::new("classic:resource.mana").unwrap(),
            pack_id: ContentId::new("pack:classic").unwrap(),
            name: "Mana".to_string(),
            description: String::new(),
            icon: None,
            tags: Vec::new(),
        },
    }
}

fn test_aura() -> AuraDefinition {
    AuraDefinition {
        metadata: ContentMetadata {
            id: ContentId::new("classic:aura.test").unwrap(),
            pack_id: ContentId::new("pack:classic").unwrap(),
            name: "Test Aura".to_string(),
            description: String::new(),
            icon: None,
            tags: Vec::new(),
        },
        max_stacks: 5,
        stacking_policy: AuraStackingPolicy::AddStacks,
        stacking_scope: AuraStackingScope::Shared,
        refresh_policy: AuraRefreshPolicy::ResetDuration,
        periodic_effects: Vec::new(),
    }
}

fn test_creature() -> CreatureDefinition {
    CreatureDefinition {
        metadata: ContentMetadata {
            id: ContentId::new("classic:creature.wolf").unwrap(),
            pack_id: ContentId::new("pack:classic").unwrap(),
            name: "Wolf Companion".to_string(),
            description: String::new(),
            icon: None,
            tags: vec!["summon".to_string()],
        },
        health: 150.0,
        attack_damage: 12.0,
        attack_interval_seconds: 2.0,
        melee_range: 2.5,
        aggro_range: 15.0,
        leash_range: 40.0,
        movement_speed: 5.0,
    }
}

mod validation_tests;
mod registration_tests;
mod serialization_tests;
