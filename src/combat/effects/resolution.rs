use std::collections::HashSet;
use std::fmt::{Display, Formatter};

use bevy::prelude::Entity;

use crate::content::{
    ContentId,
    EffectDefinition,
    EffectKind,
    EffectTarget,
    EffectTrigger,
    EffectValidationError,
};

use super::magnitude::calculate_signed_effect_magnitude;
use crate::combat::{
    calculate_effect_magnitude,
    CombatAttributes,
    DamageRequest,
    EffectMagnitudeError,
    HealingRequest,
    ResourceOperation,
    ResourceRequest,
};

#[derive(Clone, Debug, PartialEq)]
pub enum EffectTargetSelection {
    Caster,
    Selected(Entity),
    Area(Vec<Entity>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedEffect {
    pub effect: ContentId,
    pub trigger: EffectTrigger,
    pub source: Option<Entity>,
    pub target: Entity,
    pub operation: ResolvedEffectOperation,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ResolvedEffectOperation {
    Damage(DamageRequest),
    Heal(HealingRequest),
    GainResource(ResourceRequest),
    SpendResource(ResourceRequest),
    ApplyAura {
        aura: ContentId,
        stacks: u32,
        duration_seconds: Option<f32>,
    },
    RemoveAura {
        aura: ContentId,
        stacks: Option<u32>,
    },
    Summon {
        creature: ContentId,
        count: u32,
        duration_seconds: Option<f32>,
    },
    ModifyThreat {
        amount: f32,
    },
    Interrupt,
    Displacement {
        distance: f32,
    },
    Custom {
        handler: ContentId,
    },
}

pub fn resolve_effect(
    definition: &EffectDefinition,
    source: Option<Entity>,
    target_selection: EffectTargetSelection,
    source_attributes: &CombatAttributes,
) -> Result<Vec<ResolvedEffect>, EffectResolutionError> {
    definition
        .validate()
        .map_err(EffectResolutionError::InvalidDefinition)?;

    let targets = resolve_targets(
        &definition.target,
        source,
        target_selection,
    )?;

    let mut resolved_effects = Vec::with_capacity(targets.len());

    for target in targets {
        let operation = resolve_operation(
            &definition.kind,
            source,
            target,
            source_attributes,
        )?;

        resolved_effects.push(ResolvedEffect {
            effect: definition.metadata.id.clone(),
            trigger: definition.trigger,
            source,
            target,
            operation,
        });
    }

    Ok(resolved_effects)
}

fn resolve_targets(
    effect_target: &EffectTarget,
    source: Option<Entity>,
    selection: EffectTargetSelection,
) -> Result<Vec<Entity>, EffectResolutionError> {
    match (effect_target, selection) {
        (EffectTarget::Caster, EffectTargetSelection::Caster) => {
            let caster = source.ok_or(
                EffectResolutionError::MissingSourceForCaster,
            )?;

            Ok(vec![caster])
        }
        (
            EffectTarget::SelectedTarget,
            EffectTargetSelection::Selected(target),
        ) => Ok(vec![target]),
        (
            EffectTarget::AlliesInRadius { .. }
            | EffectTarget::EnemiesInRadius { .. }
            | EffectTarget::UnitsInRadius { .. },
            EffectTargetSelection::Area(targets),
        ) => {
            let mut unique_targets = HashSet::new();

            for target in &targets {
                if !unique_targets.insert(*target) {
                    return Err(
                        EffectResolutionError::DuplicateAreaTarget,
                    );
                }
            }

            Ok(targets)
        }
        _ => Err(EffectResolutionError::TargetSelectionMismatch),
    }
}

fn resolve_operation(
    kind: &EffectKind,
    source: Option<Entity>,
    target: Entity,
    source_attributes: &CombatAttributes,
) -> Result<ResolvedEffectOperation, EffectResolutionError> {
    match kind {
        EffectKind::Damage { amount, school } => {
            let amount = calculate_effect_magnitude(
                amount,
                source_attributes,
            )
            .map_err(EffectResolutionError::InvalidMagnitude)?;

            Ok(ResolvedEffectOperation::Damage(DamageRequest {
                source,
                target,
                base_amount: amount,
                school: school.clone(),
            }))
        }
        EffectKind::Heal { amount } => {
            let amount = calculate_effect_magnitude(
                amount,
                source_attributes,
            )
            .map_err(EffectResolutionError::InvalidMagnitude)?;

            Ok(ResolvedEffectOperation::Heal(HealingRequest {
                source,
                target,
                base_amount: amount,
            }))
        }
        EffectKind::GainResource { resource, amount } => {
            let amount = calculate_effect_magnitude(
                amount,
                source_attributes,
            )
            .map_err(EffectResolutionError::InvalidMagnitude)?;

            Ok(ResolvedEffectOperation::GainResource(ResourceRequest {
                source,
                target,
                resource: resource.clone(),
                operation: ResourceOperation::Gain,
                base_amount: amount,
            }))
        }
        EffectKind::SpendResource { resource, amount } => {
            let amount = calculate_effect_magnitude(
                amount,
                source_attributes,
            )
            .map_err(EffectResolutionError::InvalidMagnitude)?;

            Ok(ResolvedEffectOperation::SpendResource(ResourceRequest {
                source,
                target,
                resource: resource.clone(),
                operation: ResourceOperation::Spend,
                base_amount: amount,
            }))
        }
        EffectKind::ApplyAura {
            aura,
            stacks,
            duration_seconds,
        } => Ok(ResolvedEffectOperation::ApplyAura {
            aura: aura.clone(),
            stacks: *stacks,
            duration_seconds: *duration_seconds,
        }),
        EffectKind::RemoveAura { aura, stacks } => {
            Ok(ResolvedEffectOperation::RemoveAura {
                aura: aura.clone(),
                stacks: *stacks,
            })
        }
        EffectKind::Summon {
            creature,
            count,
            duration_seconds,
        } => Ok(ResolvedEffectOperation::Summon {
            creature: creature.clone(),
            count: *count,
            duration_seconds: *duration_seconds,
        }),
        EffectKind::ModifyThreat { amount } => {
            let amount = calculate_signed_effect_magnitude(
                amount,
                source_attributes,
            )
            .map_err(EffectResolutionError::InvalidMagnitude)?;

            Ok(ResolvedEffectOperation::ModifyThreat {
                amount,
            })
        }
        EffectKind::Interrupt => {
            Ok(ResolvedEffectOperation::Interrupt)
        }
        EffectKind::Displacement { distance } => {
            Ok(ResolvedEffectOperation::Displacement {
                distance: *distance,
            })
        }
        EffectKind::Custom { handler } => {
            Ok(ResolvedEffectOperation::Custom {
                handler: handler.clone(),
            })
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectResolutionError {
    InvalidDefinition(EffectValidationError),
    InvalidMagnitude(EffectMagnitudeError),
    MissingSourceForCaster,
    TargetSelectionMismatch,
    DuplicateAreaTarget,
}

impl Display for EffectResolutionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidDefinition(error) => {
                write!(formatter, "Definizione dell'effetto non valida: {error}")
            }
            Self::InvalidMagnitude(error) => {
                write!(formatter, "Calcolo della magnitudine non riuscito: {error}")
            }
            Self::MissingSourceForCaster => {
                formatter.write_str(
                    "Un effetto diretto al caster richiede una sorgente",
                )
            }
            Self::TargetSelectionMismatch => {
                formatter.write_str(
                    "La selezione dei bersagli non corrisponde al tipo di effetto",
                )
            }
            Self::DuplicateAreaTarget => {
                formatter.write_str(
                    "La selezione ad area contiene bersagli duplicati",
                )
            }
        }
    }
}

impl std::error::Error for EffectResolutionError {}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::content::{
        ContentMetadata,
        EffectMagnitude,
        EffectScaling,
    };

    fn source() -> Entity {
        Entity::from_bits(1)
    }

    fn target() -> Entity {
        Entity::from_bits(2)
    }

    fn target_two() -> Entity {
        Entity::from_bits(3)
    }

    fn content_id(value: &str) -> ContentId {
        ContentId::new(format!("classic_1_12_1:{value}")).unwrap()
    }

    fn valid_effect(
        target: EffectTarget,
        kind: EffectKind,
    ) -> EffectDefinition {
        EffectDefinition {
            metadata: ContentMetadata {
                id: content_id("effect.test"),
                pack_id: ContentId::new("pack:classic_1_12_1").unwrap(),
                name: "Test Effect".to_string(),
                description: String::new(),
                icon: None,
                tags: Vec::new(),
            },
            trigger: EffectTrigger::OnCast,
            target,
            kind,
        }
    }

    fn magnitude(base: f32) -> EffectMagnitude {
        EffectMagnitude {
            base,
            scaling: Vec::new(),
        }
    }

    fn attributes_with_spell_power(value: f32) -> CombatAttributes {
        let mut attributes = CombatAttributes::default();

        attributes
            .set(content_id("stat.spell_power"), value)
            .unwrap();

        attributes
    }

    #[test]
    fn caster_effect_resolves_to_source() {
        let definition = valid_effect(
            EffectTarget::Caster,
            EffectKind::Interrupt,
        );

        let result = resolve_effect(
            &definition,
            Some(source()),
            EffectTargetSelection::Caster,
            &CombatAttributes::default(),
        )
        .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].target, source());
        assert_eq!(result[0].source, Some(source()));
        assert_eq!(
            result[0].operation,
            ResolvedEffectOperation::Interrupt
        );
    }

    #[test]
    fn selected_target_effect_resolves_to_selected_entity() {
        let definition = valid_effect(
            EffectTarget::SelectedTarget,
            EffectKind::Interrupt,
        );

        let result = resolve_effect(
            &definition,
            Some(source()),
            EffectTargetSelection::Selected(target()),
            &CombatAttributes::default(),
        )
        .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].target, target());
    }

    #[test]
    fn area_effect_resolves_all_unique_targets() {
        let definition = valid_effect(
            EffectTarget::EnemiesInRadius { radius: 10.0 },
            EffectKind::Interrupt,
        );

        let result = resolve_effect(
            &definition,
            Some(source()),
            EffectTargetSelection::Area(vec![target(), target_two()]),
            &CombatAttributes::default(),
        )
        .unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].target, target());
        assert_eq!(result[1].target, target_two());
    }

    #[test]
    fn area_effect_accepts_empty_target_list() {
        let definition = valid_effect(
            EffectTarget::UnitsInRadius { radius: 10.0 },
            EffectKind::Interrupt,
        );

        let result = resolve_effect(
            &definition,
            Some(source()),
            EffectTargetSelection::Area(Vec::new()),
            &CombatAttributes::default(),
        )
        .unwrap();

        assert!(result.is_empty());
    }

    #[test]
    fn area_effect_rejects_duplicate_targets() {
        let definition = valid_effect(
            EffectTarget::AlliesInRadius { radius: 10.0 },
            EffectKind::Interrupt,
        );

        assert_eq!(
            resolve_effect(
                &definition,
                Some(source()),
                EffectTargetSelection::Area(vec![target(), target()]),
                &CombatAttributes::default(),
            ),
            Err(EffectResolutionError::DuplicateAreaTarget)
        );
    }

    #[test]
    fn effect_rejects_mismatched_target_selection() {
        let definition = valid_effect(
            EffectTarget::SelectedTarget,
            EffectKind::Interrupt,
        );

        assert_eq!(
            resolve_effect(
                &definition,
                Some(source()),
                EffectTargetSelection::Caster,
                &CombatAttributes::default(),
            ),
            Err(EffectResolutionError::TargetSelectionMismatch)
        );
    }

    #[test]
    fn caster_effect_requires_source() {
        let definition = valid_effect(
            EffectTarget::Caster,
            EffectKind::Interrupt,
        );

        assert_eq!(
            resolve_effect(
                &definition,
                None,
                EffectTargetSelection::Caster,
                &CombatAttributes::default(),
            ),
            Err(EffectResolutionError::MissingSourceForCaster)
        );
    }

    #[test]
    fn damage_effect_builds_scaled_damage_request() {
        let definition = valid_effect(
            EffectTarget::SelectedTarget,
            EffectKind::Damage {
                amount: EffectMagnitude {
                    base: 20.0,
                    scaling: vec![EffectScaling {
                        stat: content_id("stat.spell_power"),
                        coefficient: 0.5,
                    }],
                },
                school: content_id("school.fire"),
            },
        );

        let result = resolve_effect(
            &definition,
            Some(source()),
            EffectTargetSelection::Selected(target()),
            &attributes_with_spell_power(100.0),
        )
        .unwrap();

        assert_eq!(
            result[0].operation,
            ResolvedEffectOperation::Damage(DamageRequest {
                source: Some(source()),
                target: target(),
                base_amount: 70.0,
                school: content_id("school.fire"),
            })
        );
    }

    #[test]
    fn healing_effect_builds_healing_request() {
        let definition = valid_effect(
            EffectTarget::SelectedTarget,
            EffectKind::Heal {
                amount: magnitude(50.0),
            },
        );

        let result = resolve_effect(
            &definition,
            Some(source()),
            EffectTargetSelection::Selected(target()),
            &CombatAttributes::default(),
        )
        .unwrap();

        assert_eq!(
            result[0].operation,
            ResolvedEffectOperation::Heal(HealingRequest {
                source: Some(source()),
                target: target(),
                base_amount: 50.0,
            })
        );
    }

    #[test]
    fn resource_gain_effect_builds_gain_request() {
        let definition = valid_effect(
            EffectTarget::Caster,
            EffectKind::GainResource {
                resource: content_id("resource.mana"),
                amount: magnitude(30.0),
            },
        );

        let result = resolve_effect(
            &definition,
            Some(source()),
            EffectTargetSelection::Caster,
            &CombatAttributes::default(),
        )
        .unwrap();

        assert_eq!(
            result[0].operation,
            ResolvedEffectOperation::GainResource(ResourceRequest {
                source: Some(source()),
                target: source(),
                resource: content_id("resource.mana"),
                operation: ResourceOperation::Gain,
                base_amount: 30.0,
            })
        );
    }

    #[test]
    fn resource_spending_effect_builds_spend_request() {
        let definition = valid_effect(
            EffectTarget::Caster,
            EffectKind::SpendResource {
                resource: content_id("resource.mana"),
                amount: magnitude(20.0),
            },
        );

        let result = resolve_effect(
            &definition,
            Some(source()),
            EffectTargetSelection::Caster,
            &CombatAttributes::default(),
        )
        .unwrap();

        assert_eq!(
            result[0].operation,
            ResolvedEffectOperation::SpendResource(ResourceRequest {
                source: Some(source()),
                target: source(),
                resource: content_id("resource.mana"),
                operation: ResourceOperation::Spend,
                base_amount: 20.0,
            })
        );
    }

    #[test]
    fn aura_effect_preserves_stacks_and_duration() {
        let definition = valid_effect(
            EffectTarget::SelectedTarget,
            EffectKind::ApplyAura {
                aura: content_id("aura.test"),
                stacks: 3,
                duration_seconds: Some(12.0),
            },
        );

        let result = resolve_effect(
            &definition,
            Some(source()),
            EffectTargetSelection::Selected(target()),
            &CombatAttributes::default(),
        )
        .unwrap();

        assert_eq!(
            result[0].operation,
            ResolvedEffectOperation::ApplyAura {
                aura: content_id("aura.test"),
                stacks: 3,
                duration_seconds: Some(12.0),
            }
        );
    }

    #[test]
    fn summon_effect_preserves_creature_and_count() {
        let definition = valid_effect(
            EffectTarget::Caster,
            EffectKind::Summon {
                creature: content_id("creature.imp"),
                count: 2,
                duration_seconds: Some(30.0),
            },
        );

        let result = resolve_effect(
            &definition,
            Some(source()),
            EffectTargetSelection::Caster,
            &CombatAttributes::default(),
        )
        .unwrap();

        assert_eq!(
            result[0].operation,
            ResolvedEffectOperation::Summon {
                creature: content_id("creature.imp"),
                count: 2,
                duration_seconds: Some(30.0),
            }
        );
    }

    #[test]
    fn threat_effect_applies_magnitude_scaling() {
        let definition = valid_effect(
            EffectTarget::SelectedTarget,
            EffectKind::ModifyThreat {
                amount: EffectMagnitude {
                    base: 10.0,
                    scaling: vec![EffectScaling {
                        stat: content_id("stat.spell_power"),
                        coefficient: 0.5,
                    }],
                },
            },
        );

        let result = resolve_effect(
            &definition,
            Some(source()),
            EffectTargetSelection::Selected(target()),
            &attributes_with_spell_power(100.0),
        )
        .unwrap();

        assert_eq!(
            result[0].operation,
            ResolvedEffectOperation::ModifyThreat {
                amount: 60.0,
            }
        );
    }

    #[test]
    fn threat_effect_resolves_negative_scaled_magnitude() {
        let definition = valid_effect(
            EffectTarget::SelectedTarget,
            EffectKind::ModifyThreat {
                amount: EffectMagnitude {
                    base: -10.0,
                    scaling: vec![EffectScaling {
                        stat: content_id("stat.spell_power"),
                        coefficient: -0.5,
                    }],
                },
            },
        );

        let result = resolve_effect(
            &definition,
            Some(source()),
            EffectTargetSelection::Selected(target()),
            &attributes_with_spell_power(100.0),
        )
        .unwrap();

        assert_eq!(
            result[0].operation,
            ResolvedEffectOperation::ModifyThreat {
                amount: -60.0,
            }
        );
    }

    #[test]
    fn displacement_effect_preserves_distance() {
        let definition = valid_effect(
            EffectTarget::SelectedTarget,
            EffectKind::Displacement {
                distance: 5.0,
            },
        );

        let result = resolve_effect(
            &definition,
            Some(source()),
            EffectTargetSelection::Selected(target()),
            &CombatAttributes::default(),
        )
        .unwrap();

        assert_eq!(
            result[0].operation,
            ResolvedEffectOperation::Displacement {
                distance: 5.0,
            }
        );
    }

    #[test]
    fn custom_effect_preserves_handler_id() {
        let definition = valid_effect(
            EffectTarget::SelectedTarget,
            EffectKind::Custom {
                handler: content_id("handler.custom"),
            },
        );

        let result = resolve_effect(
            &definition,
            Some(source()),
            EffectTargetSelection::Selected(target()),
            &CombatAttributes::default(),
        )
        .unwrap();

        assert_eq!(
            result[0].operation,
            ResolvedEffectOperation::Custom {
                handler: content_id("handler.custom"),
            }
        );
    }
}
