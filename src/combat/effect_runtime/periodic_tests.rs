
use bevy::prelude::Entity;

use crate::content::{
    ContentId,
    PeriodicEffectStackScaling,
};

use super::{
    apply_periodic_stack_scaling,
    EffectQueueError,
    ResolvedEffect,
    ResolvedEffectOperation,
};

use super::super::{
    EffectTrigger,
    HealingRequest,
};

fn content_id(value: &str) -> ContentId {
    ContentId::new(
        format!("classic_1_12_1:{value}"),
    )
    .unwrap()
}

fn periodic_effect_id() -> ContentId {
    content_id("effect.periodic_test")
}

fn source() -> Entity {
    Entity::from_bits(1)
}

fn target() -> Entity {
    Entity::from_bits(2)
}

fn heal_effect(base_amount: f32) -> ResolvedEffect {
    ResolvedEffect {
        effect: periodic_effect_id(),
        trigger: EffectTrigger::OnPeriodicTick,
        source: Some(source()),
        target: target(),
        operation: ResolvedEffectOperation::Heal(
            HealingRequest {
                source: Some(source()),
                target: target(),
                base_amount,
            },
        ),
    }
}

fn healing_amount(effect: &ResolvedEffect) -> f32 {
    match &effect.operation {
        ResolvedEffectOperation::Heal(request) => {
            request.base_amount
        }
        _ => panic!("L'effetto di test non è una guarigione"),
    }
}

#[test]
fn linear_periodic_scaling_multiplies_amount_by_stacks() {
    let mut effect = heal_effect(12.5);

    let result = apply_periodic_stack_scaling(
        &mut effect,
        3,
        PeriodicEffectStackScaling::Linear,
    );

    assert_eq!(result, Ok(()));
    assert_eq!(healing_amount(&effect), 37.5);
}

#[test]
fn disabled_periodic_scaling_preserves_base_amount() {
    let mut effect = heal_effect(12.5);

    let result = apply_periodic_stack_scaling(
        &mut effect,
        5,
        PeriodicEffectStackScaling::None,
    );

    assert_eq!(result, Ok(()));
    assert_eq!(healing_amount(&effect), 12.5);
}

#[test]
fn periodic_scaling_rejects_zero_stacks_without_mutating_effect() {
    let mut effect = heal_effect(12.5);

    let result = apply_periodic_stack_scaling(
        &mut effect,
        0,
        PeriodicEffectStackScaling::Linear,
    );

    assert_eq!(
        result,
        Err(EffectQueueError::ZeroPeriodicStacks)
    );

    assert_eq!(healing_amount(&effect), 12.5);
}

#[test]
fn periodic_scaling_rejects_non_finite_result_without_mutating_effect() {
    let mut effect = heal_effect(f32::MAX);

    let result = apply_periodic_stack_scaling(
        &mut effect,
        2,
        PeriodicEffectStackScaling::Linear,
    );

    assert_eq!(
        result,
        Err(EffectQueueError::InvalidPeriodicMagnitude(
            periodic_effect_id()
        ))
    );

    assert_eq!(healing_amount(&effect), f32::MAX);
}

#[test]
fn periodic_scaling_does_not_modify_non_numeric_effects() {
    let mut effect = ResolvedEffect {
        effect: periodic_effect_id(),
        trigger: EffectTrigger::OnPeriodicTick,
        source: Some(source()),
        target: target(),
        operation: ResolvedEffectOperation::Interrupt,
    };

    let result = apply_periodic_stack_scaling(
        &mut effect,
        4,
        PeriodicEffectStackScaling::Linear,
    );

    assert_eq!(result, Ok(()));

    assert_eq!(
        effect.operation,
        ResolvedEffectOperation::Interrupt
    );
}
