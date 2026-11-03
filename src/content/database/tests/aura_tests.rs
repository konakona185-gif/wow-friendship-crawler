use super::{
    aura,
    database_with_classic_pack,
    metadata,
};
use super::super::ContentDatabaseError;

use crate::content::{
    ContentId,
    EffectDefinition,
    EffectKind,
    EffectMagnitude,
    EffectTarget,
    EffectTrigger,
    PeriodicAuraEffect,
    PeriodicEffectStackScaling,
};

fn periodic_heal_effect() -> EffectDefinition {
    EffectDefinition {
        metadata: metadata(
            "classic:effect.periodic_heal",
            "Periodic Heal",
        ),
        trigger: EffectTrigger::OnPeriodicTick,
        target: EffectTarget::SelectedTarget,
        kind: EffectKind::Heal {
            amount: EffectMagnitude {
                base: 10.0,
                scaling: Vec::new(),
            },
        },
    }
}

#[test]
fn database_rejects_auras_with_unknown_periodic_effects() {
    let mut database = database_with_classic_pack();
    let mut aura = aura();

    let aura_id = aura.metadata.id.clone();
    let unknown_effect =
        ContentId::new("classic:effect.periodic_missing").unwrap();

    aura.periodic_effects.push(PeriodicAuraEffect {
        effect: unknown_effect.clone(),
        interval_seconds: 3.0,
        stack_scaling: PeriodicEffectStackScaling::None,
    });

    database.register_aura(aura).unwrap();

    assert_eq!(
        database.validate(),
        Err(ContentDatabaseError::MissingReference {
            owner: aura_id,
            reference: unknown_effect,
        })
    );
}

#[test]
fn database_accepts_auras_with_registered_periodic_effects() {
    let mut database = database_with_classic_pack();
    let effect = periodic_heal_effect();
    let effect_id = effect.metadata.id.clone();

    database.register_effect(effect).unwrap();

    let mut aura = aura();

    aura.periodic_effects.push(PeriodicAuraEffect {
        effect: effect_id,
        interval_seconds: 3.0,
        stack_scaling: PeriodicEffectStackScaling::None,
    });

    database.register_aura(aura).unwrap();

    assert!(database.validate().is_ok());
    assert_eq!(database.aura_count(), 1);
    assert_eq!(database.effect_count(), 1);
}
