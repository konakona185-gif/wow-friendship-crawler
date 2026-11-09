use bevy::prelude::Entity;

use crate::content::{ContentId, PeriodicEffectStackScaling};

use super::super::{
    apply_periodic_stack_scaling, EffectQueueError, ResolvedEffect, ResolvedEffectOperation,
};

use crate::combat::{
    DamageRequest, EffectTrigger, HealingRequest, ResourceOperation, ResourceRequest,
};

fn content_id(value: &str) -> ContentId {
    ContentId::new(format!("classic_1_12_1:{value}")).unwrap()
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
        operation: ResolvedEffectOperation::Heal(HealingRequest {
            source: Some(source()),
            target: target(),
            base_amount,
        }),
    }
}

fn damage_effect(base_amount: f32) -> ResolvedEffect {
    ResolvedEffect {
        effect: periodic_effect_id(),
        trigger: EffectTrigger::OnPeriodicTick,
        source: Some(source()),
        target: target(),
        operation: ResolvedEffectOperation::Damage(DamageRequest {
            source: Some(source()),
            target: target(),
            base_amount,
            school: content_id("school.shadow"),
        }),
    }
}

fn resource_effect(base_amount: f32, operation: ResourceOperation) -> ResolvedEffect {
    let request = ResourceRequest {
        source: Some(source()),
        target: target(),
        resource: content_id("resource.mana"),
        operation,
        base_amount,
    };

    let operation = match operation {
        ResourceOperation::Gain => ResolvedEffectOperation::GainResource(request),
        ResourceOperation::Spend => ResolvedEffectOperation::SpendResource(request),
    };

    ResolvedEffect {
        effect: periodic_effect_id(),
        trigger: EffectTrigger::OnPeriodicTick,
        source: Some(source()),
        target: target(),
        operation,
    }
}

fn threat_effect(amount: f32) -> ResolvedEffect {
    ResolvedEffect {
        effect: periodic_effect_id(),
        trigger: EffectTrigger::OnPeriodicTick,
        source: Some(source()),
        target: target(),
        operation: ResolvedEffectOperation::ModifyThreat { amount },
    }
}

fn healing_amount(effect: &ResolvedEffect) -> f32 {
    match &effect.operation {
        ResolvedEffectOperation::Heal(request) => request.base_amount,
        _ => panic!("L'effetto di test non è una guarigione"),
    }
}

#[test]
fn linear_periodic_scaling_multiplies_amount_by_stacks() {
    let mut effect = heal_effect(12.5);

    let result = apply_periodic_stack_scaling(&mut effect, 3, PeriodicEffectStackScaling::Linear);

    assert_eq!(result, Ok(()));
    assert_eq!(healing_amount(&effect), 37.5);
}

#[test]
fn linear_periodic_scaling_multiplies_damage_by_stacks() {
    let mut effect = damage_effect(20.0);

    let result = apply_periodic_stack_scaling(&mut effect, 4, PeriodicEffectStackScaling::Linear);

    assert_eq!(result, Ok(()));

    match &effect.operation {
        ResolvedEffectOperation::Damage(request) => {
            assert_eq!(request.base_amount, 80.0);
        }
        _ => panic!("L'effetto di test non è un danno"),
    }
}

#[test]
fn linear_periodic_scaling_multiplies_resource_gain_by_stacks() {
    let mut effect = resource_effect(10.0, ResourceOperation::Gain);

    let result = apply_periodic_stack_scaling(&mut effect, 3, PeriodicEffectStackScaling::Linear);

    assert_eq!(result, Ok(()));

    match &effect.operation {
        ResolvedEffectOperation::GainResource(request) => {
            assert_eq!(request.base_amount, 30.0);
            assert_eq!(request.operation, ResourceOperation::Gain);
        }
        _ => panic!("L'effetto di test non è un guadagno di risorsa"),
    }
}

#[test]
fn linear_periodic_scaling_multiplies_resource_spending_by_stacks() {
    let mut effect = resource_effect(10.0, ResourceOperation::Spend);

    let result = apply_periodic_stack_scaling(&mut effect, 3, PeriodicEffectStackScaling::Linear);

    assert_eq!(result, Ok(()));

    match &effect.operation {
        ResolvedEffectOperation::SpendResource(request) => {
            assert_eq!(request.base_amount, 30.0);
            assert_eq!(request.operation, ResourceOperation::Spend);
        }
        _ => panic!("L'effetto di test non è una spesa di risorsa"),
    }
}

#[test]
fn linear_periodic_scaling_multiplies_threat_by_stacks() {
    let mut effect = threat_effect(15.0);

    let result = apply_periodic_stack_scaling(&mut effect, 4, PeriodicEffectStackScaling::Linear);

    assert_eq!(result, Ok(()));

    match &effect.operation {
        ResolvedEffectOperation::ModifyThreat { amount } => {
            assert_eq!(*amount, 60.0);
        }
        _ => panic!("L'effetto di test non modifica la threat"),
    }
}

#[test]
fn disabled_periodic_scaling_preserves_base_amount() {
    let mut effect = heal_effect(12.5);

    let result = apply_periodic_stack_scaling(&mut effect, 5, PeriodicEffectStackScaling::None);

    assert_eq!(result, Ok(()));
    assert_eq!(healing_amount(&effect), 12.5);
}

#[test]
fn periodic_scaling_rejects_zero_stacks_without_mutating_effect() {
    let mut effect = heal_effect(12.5);

    let result = apply_periodic_stack_scaling(&mut effect, 0, PeriodicEffectStackScaling::Linear);

    assert_eq!(result, Err(EffectQueueError::ZeroPeriodicStacks));

    assert_eq!(healing_amount(&effect), 12.5);
}

#[test]
fn periodic_scaling_rejects_non_finite_result_without_mutating_effect() {
    let mut effect = heal_effect(f32::MAX);

    let result = apply_periodic_stack_scaling(&mut effect, 2, PeriodicEffectStackScaling::Linear);

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

    let result = apply_periodic_stack_scaling(&mut effect, 4, PeriodicEffectStackScaling::Linear);

    assert_eq!(result, Ok(()));

    assert_eq!(effect.operation, ResolvedEffectOperation::Interrupt);
}
