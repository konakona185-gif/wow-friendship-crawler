use super::*;

fn source() -> Entity {
    Entity::from_bits(1)
}

fn target() -> Entity {
    Entity::from_bits(2)
}

fn content_id(value: &str) -> ContentId {
    ContentId::new(format!("classic_1_12_1:{value}")).unwrap()
}

fn valid_context(trigger: EffectTrigger) -> CombatEventContext {
    CombatEventContext {
        trigger,
        source: Some(source()),
        target: Some(target()),
        ability: Some(content_id("ability.test")),
        effect: None,
        outcome: None,
    }
}

fn damage_outcome() -> CombatEventOutcome {
    let request = super::super::super::DamageRequest {
        source: Some(source()),
        target: target(),
        base_amount: 100.0,
        school: content_id("school.physical"),
    };

    CombatEventOutcome::Damage(
        DamageResolution::new(
            request,
            super::super::super::DamageMitigationResult::new(100.0, 25.0).unwrap(),
        )
        .unwrap(),
    )
}

fn healing_outcome() -> CombatEventOutcome {
    let request = super::super::super::HealingRequest {
        source: Some(source()),
        target: target(),
        base_amount: 100.0,
    };

    CombatEventOutcome::Healing(HealingResolution::new(request, 75.0).unwrap())
}

fn resource_outcome() -> CombatEventOutcome {
    let request = super::super::super::ResourceRequest {
        source: Some(source()),
        target: target(),
        resource: content_id("resource.mana"),
        operation: super::super::super::ResourceOperation::Gain,
        base_amount: 50.0,
    };

    CombatEventOutcome::Resource(ResourceResolution::new(request, 30.0).unwrap())
}

fn aura_applied_outcome(total_stacks: u32) -> CombatEventOutcome {
    CombatEventOutcome::AuraApplied {
        aura: content_id("aura.test"),
        total_stacks,
    }
}

fn aura_removed_outcome(removed_stacks: u32) -> CombatEventOutcome {
    CombatEventOutcome::AuraRemoved {
        aura: content_id("aura.test"),
        removed_stacks,
    }
}

fn aura_expired_outcome(stacks: u32) -> CombatEventOutcome {
    CombatEventOutcome::AuraExpired {
        aura: content_id("aura.test"),
        stacks,
    }
}

fn periodic_tick_outcome(stacks: u32) -> CombatEventOutcome {
    CombatEventOutcome::PeriodicTick {
        aura: content_id("aura.test"),
        stacks,
        stack_scaling: PeriodicEffectStackScaling::Linear,
    }
}

fn valid_periodic_tick_context() -> CombatEventContext {
    let mut context = valid_context(EffectTrigger::OnPeriodicTick);

    context.effect = Some(content_id("effect.periodic_test"));
    context.outcome = Some(periodic_tick_outcome(2));

    context
}

mod context_validation_tests;
mod queue_tests;
mod subscription_tests;
