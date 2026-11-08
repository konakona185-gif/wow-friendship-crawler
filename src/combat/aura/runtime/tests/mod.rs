use super::*;
use crate::content::{
    AuraRefreshPolicy,
    AuraStackingPolicy,
    AuraStackingScope,
    ContentId,
    ContentMetadata,
    PeriodicAuraEffect,
    PeriodicEffectStackScaling,
};

fn aura_definition(
    stacking_policy: AuraStackingPolicy,
    max_stacks: u32,
) -> AuraDefinition {
    AuraDefinition {
        metadata: ContentMetadata {
            id: aura_id(),
            pack_id: ContentId::new("pack:test").unwrap(),
            name: "Test Aura".to_string(),
            description: String::new(),
            icon: None,
            tags: Vec::new(),
        },
        max_stacks,
        stacking_policy,
        stacking_scope: AuraStackingScope::Shared,
        refresh_policy: AuraRefreshPolicy::ResetDuration,
        periodic_effects: Vec::new(),
    }
}

fn periodic_aura_definition(
    stacking_policy: AuraStackingPolicy,
    max_stacks: u32,
    interval_seconds: f32,
    stack_scaling: PeriodicEffectStackScaling,
) -> AuraDefinition {
    let mut definition = aura_definition(
        stacking_policy,
        max_stacks,
    );

    definition.periodic_effects.push(PeriodicAuraEffect {
        effect: periodic_effect_id(),
        interval_seconds,
        stack_scaling,
    });

    definition
}

fn aura_id() -> ContentId {
    ContentId::new("test:aura.test").unwrap()
}

fn periodic_effect_id() -> ContentId {
    ContentId::new("test:effect.periodic_test").unwrap()
}

fn source() -> Entity {
    Entity::from_raw_u32(1).unwrap()
}

fn other_source() -> Entity {
    Entity::from_raw_u32(2).unwrap()
}

fn third_source() -> Entity {
    Entity::from_raw_u32(3).unwrap()
}

mod stacking_tests;
mod periodic_tests;
mod lifecycle_validation_tests;
