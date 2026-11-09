use std::fmt::{Display, Formatter};

use bevy::prelude::Entity;

use crate::content::{ContentDatabase, ContentId, ContentIdError};

use crate::combat::{
    CombatAttributes, CombatLevel, CombatResources, DamageResolution, EnemyThreat,
    HealingRequestError, HealingResolution, HealingResolutionError, HealthPool, HealthPoolError,
    ResolvedEffect, ResolvedEffectOperation, ResourceApplicationError, ResourceOperation,
    ResourceRequest, ResourceResolution,
};

pub struct EffectExecutionContext<'a> {
    pub database: &'a ContentDatabase,
    pub attacker_level: Option<&'a CombatLevel>,
    pub target_attributes: Option<&'a CombatAttributes>,
    pub target_health: Option<&'a mut HealthPool>,
    pub target_resources: Option<&'a mut CombatResources>,
    pub target_threat: Option<&'a mut EnemyThreat>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum EffectExecutionResult {
    Damage {
        resolution: DamageResolution,
        applied_amount: f32,
    },
    Healing {
        resolution: HealingResolution,
        applied_amount: f32,
    },
    Resource(ResourceResolution),
    Threat {
        modified_amount: f32,
    },
}

pub fn execute_resolved_effect(
    effect: &ResolvedEffect,
    mut context: EffectExecutionContext<'_>,
) -> Result<EffectExecutionResult, EffectExecutionError> {
    match &effect.operation {
        ResolvedEffectOperation::Damage(request) => {
            if request.source != effect.source || request.target != effect.target {
                return Err(EffectExecutionError::ContextMismatch);
            }

            let school = context
                .database
                .damage_school(&request.school)
                .ok_or_else(|| EffectExecutionError::MissingDamageSchool(request.school.clone()))?;

            let armor_attribute = ContentId::new("classic_1_12_1:stat.armor")
                .map_err(EffectExecutionError::InvalidArmorAttributeId)?;

            if context
                .database
                .combat_attribute(&armor_attribute)
                .is_none()
            {
                return Err(EffectExecutionError::MissingArmorAttribute(armor_attribute));
            }

            let attacker_level = context
                .attacker_level
                .ok_or(EffectExecutionError::MissingAttackerLevel)?;

            let target_attributes = context
                .target_attributes
                .ok_or(EffectExecutionError::MissingTargetAttributes)?;

            let target_health = context
                .target_health
                .as_deref_mut()
                .ok_or(EffectExecutionError::MissingTargetHealth)?;

            let resolution = DamageResolution::from_school_mitigation(
                request.clone(),
                school,
                &armor_attribute,
                target_attributes,
                attacker_level,
            )
            .map_err(EffectExecutionError::InvalidDamageResolution)?;

            let applied_amount = target_health
                .apply_damage_resolution(&resolution)
                .map_err(EffectExecutionError::HealthApplication)?;

            Ok(EffectExecutionResult::Damage {
                resolution,
                applied_amount,
            })
        }
        ResolvedEffectOperation::Heal(request) => {
            if request.source != effect.source || request.target != effect.target {
                return Err(EffectExecutionError::ContextMismatch);
            }

            request
                .validate()
                .map_err(EffectExecutionError::InvalidHealingRequest)?;

            let target_health = context
                .target_health
                .as_deref_mut()
                .ok_or(EffectExecutionError::MissingTargetHealth)?;

            let effective_amount = target_health.remaining_capacity().min(request.base_amount);

            let resolution = HealingResolution::new(request.clone(), effective_amount)
                .map_err(EffectExecutionError::InvalidHealingResolution)?;

            let applied_amount = target_health
                .apply_healing_resolution(&resolution)
                .map_err(EffectExecutionError::HealthApplication)?;

            Ok(EffectExecutionResult::Healing {
                resolution,
                applied_amount,
            })
        }
        ResolvedEffectOperation::GainResource(request) => {
            execute_resource_request(effect, request, ResourceOperation::Gain, &mut context)
        }
        ResolvedEffectOperation::SpendResource(request) => {
            execute_resource_request(effect, request, ResourceOperation::Spend, &mut context)
        }
        ResolvedEffectOperation::ModifyThreat { amount } => {
            let source = effect
                .source
                .ok_or(EffectExecutionError::MissingSourceForThreatModification)?;

            if !amount.is_finite() {
                return Err(EffectExecutionError::InvalidThreatAmount);
            }

            let target_threat = context
                .target_threat
                .as_deref_mut()
                .ok_or(EffectExecutionError::MissingTargetThreat)?;

            target_threat.modify_threat(source, *amount);

            Ok(EffectExecutionResult::Threat {
                modified_amount: *amount,
            })
        }
        _ => Err(EffectExecutionError::UnsupportedOperation),
    }
}

fn execute_resource_request(
    effect: &ResolvedEffect,
    request: &ResourceRequest,
    expected_operation: ResourceOperation,
    context: &mut EffectExecutionContext<'_>,
) -> Result<EffectExecutionResult, EffectExecutionError> {
    if request.source != effect.source || request.target != effect.target {
        return Err(EffectExecutionError::ContextMismatch);
    }

    if request.operation != expected_operation {
        return Err(EffectExecutionError::ResourceOperationMismatch);
    }

    if context.database.resource(&request.resource).is_none() {
        return Err(EffectExecutionError::MissingResourceDefinition(
            request.resource.clone(),
        ));
    }

    let target_resources = context
        .target_resources
        .as_deref_mut()
        .ok_or(EffectExecutionError::MissingTargetResources)?;

    let resolution = target_resources
        .apply_request(request.clone())
        .map_err(EffectExecutionError::ResourceApplication)?;

    Ok(EffectExecutionResult::Resource(resolution))
}

#[derive(Clone, Debug, PartialEq)]
pub enum EffectExecutionError {
    ContextMismatch,
    MissingDamageSchool(ContentId),
    MissingArmorAttribute(ContentId),
    MissingResourceDefinition(ContentId),
    MissingAttackerLevel,
    MissingTargetAttributes,
    MissingTargetHealth,
    MissingTargetResources,
    MissingSourceForThreatModification,
    MissingTargetThreat,
    InvalidThreatAmount,
    InvalidArmorAttributeId(ContentIdError),
    InvalidDamageResolution(crate::combat::DamageResolutionError),
    InvalidHealingRequest(HealingRequestError),
    InvalidHealingResolution(HealingResolutionError),
    HealthApplication(HealthPoolError),
    ResourceOperationMismatch,
    ResourceApplication(ResourceApplicationError),
    UnsupportedOperation,
}

impl Display for EffectExecutionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ContextMismatch => formatter.write_str(
                "La sorgente o il bersaglio dell'operazione non corrispondono all'effetto",
            ),
            Self::MissingDamageSchool(id) => {
                write!(formatter, "Scuola di danno non registrata: {}", id.as_str())
            }
            Self::MissingArmorAttribute(id) => {
                write!(
                    formatter,
                    "Attributo dell'armatura non registrato: {}",
                    id.as_str()
                )
            }
            Self::MissingResourceDefinition(id) => {
                write!(
                    formatter,
                    "Definizione della risorsa non registrata: {}",
                    id.as_str()
                )
            }
            Self::MissingAttackerLevel => {
                formatter.write_str("Il calcolo del danno richiede il livello dell'attaccante")
            }
            Self::MissingTargetAttributes => {
                formatter.write_str("Il calcolo del danno richiede gli attributi del bersaglio")
            }
            Self::MissingTargetHealth => {
                formatter.write_str("Il bersaglio non possiede una HealthPool")
            }
            Self::MissingTargetResources => {
                formatter.write_str("Il bersaglio non possiede CombatResources")
            }
            Self::MissingSourceForThreatModification => {
                formatter.write_str("La modifica della minaccia richiede una sorgente")
            }
            Self::MissingTargetThreat => {
                formatter.write_str("Il bersaglio non possiede EnemyThreat")
            }
            Self::InvalidThreatAmount => {
                formatter.write_str("La quantità di minaccia da modificare non è valida")
            }
            Self::InvalidArmorAttributeId(error) => {
                write!(
                    formatter,
                    "Identificativo dell'armatura non valido: {error}"
                )
            }
            Self::InvalidDamageResolution(error) => {
                write!(formatter, "Risoluzione del danno non valida: {error}")
            }
            Self::InvalidHealingRequest(error) => {
                write!(formatter, "Richiesta di guarigione non valida: {error}")
            }
            Self::InvalidHealingResolution(error) => {
                write!(
                    formatter,
                    "Risoluzione della guarigione non valida: {error}"
                )
            }
            Self::HealthApplication(error) => {
                write!(
                    formatter,
                    "Applicazione alla HealthPool non riuscita: {error}"
                )
            }
            Self::ResourceOperationMismatch => formatter
                .write_str("Il tipo di operazione della risorsa non corrisponde all'effetto"),
            Self::ResourceApplication(error) => {
                write!(
                    formatter,
                    "Applicazione della risorsa non riuscita: {error}"
                )
            }
            Self::UnsupportedOperation => {
                formatter.write_str("L'operazione dell'effetto non è ancora supportata")
            }
        }
    }
}

impl std::error::Error for EffectExecutionError {}

#[cfg(test)]
mod tests;
