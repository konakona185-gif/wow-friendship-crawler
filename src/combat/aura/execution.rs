use std::fmt::{Display, Formatter};

use crate::content::{ContentDatabase, ContentId};

use crate::combat::{ActiveAuras, AuraRuntimeError, ResolvedEffect, ResolvedEffectOperation};

#[derive(Clone, Debug, PartialEq)]
pub(in crate::combat) enum AuraExecutionResult {
    Applied {
        aura: ContentId,
        total_stacks: u32,
    },
    Removed {
        aura: ContentId,
        removed_stacks: u32,
    },
}

pub(in crate::combat) fn execute_aura_effect(
    effect: &ResolvedEffect,
    database: &ContentDatabase,
    target_auras: Option<&mut ActiveAuras>,
) -> Result<AuraExecutionResult, AuraExecutionError> {
    match &effect.operation {
        ResolvedEffectOperation::ApplyAura {
            aura,
            stacks,
            duration_seconds,
        } => {
            let definition = database
                .aura(aura)
                .ok_or_else(|| AuraExecutionError::MissingAuraDefinition(aura.clone()))?;

            let target_auras = target_auras.ok_or(AuraExecutionError::MissingTargetAuras)?;

            target_auras
                .apply(definition, effect.source, *stacks, *duration_seconds)
                .map_err(AuraExecutionError::Application)?;

            Ok(AuraExecutionResult::Applied {
                aura: aura.clone(),
                total_stacks: target_auras.stacks(aura),
            })
        }
        ResolvedEffectOperation::RemoveAura { aura, stacks } => {
            if database.aura(aura).is_none() {
                return Err(AuraExecutionError::MissingAuraDefinition(aura.clone()));
            }

            let target_auras = target_auras.ok_or(AuraExecutionError::MissingTargetAuras)?;

            let removed_stacks = target_auras.remove(aura, *stacks);

            Ok(AuraExecutionResult::Removed {
                aura: aura.clone(),
                removed_stacks,
            })
        }
        _ => Err(AuraExecutionError::UnsupportedOperation),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(in crate::combat) enum AuraExecutionError {
    MissingAuraDefinition(ContentId),
    MissingTargetAuras,
    Application(AuraRuntimeError),
    UnsupportedOperation,
}

impl Display for AuraExecutionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingAuraDefinition(id) => {
                write!(
                    formatter,
                    "Definizione dell'aura non registrata: {}",
                    id.as_str()
                )
            }
            Self::MissingTargetAuras => {
                formatter.write_str("Il bersaglio non possiede il componente ActiveAuras")
            }
            Self::Application(error) => {
                write!(formatter, "Applicazione dell'aura non riuscita: {error:?}")
            }
            Self::UnsupportedOperation => formatter
                .write_str("L'operazione ricevuta non è un'applicazione o rimozione di un'aura"),
        }
    }
}

impl std::error::Error for AuraExecutionError {}

#[cfg(test)]
mod tests {
    use super::*;

    use bevy::prelude::Entity;

    use crate::content::{
        AuraDefinition, AuraRefreshPolicy, AuraStackingPolicy, AuraStackingScope, ContentMetadata,
        ContentPackManifest, EffectTrigger,
    };

    fn source() -> Entity {
        Entity::from_bits(1)
    }

    fn target() -> Entity {
        Entity::from_bits(2)
    }

    fn aura_definition() -> AuraDefinition {
        AuraDefinition {
            metadata: ContentMetadata {
                id: ContentId::new("classic_1_12_1:aura.test").unwrap(),
                pack_id: ContentId::new("pack:classic_1_12_1").unwrap(),
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

    fn database_with_pack() -> ContentDatabase {
        let mut database = ContentDatabase::new();

        database
            .register_pack(ContentPackManifest {
                id: ContentId::new("pack:classic_1_12_1").unwrap(),
                name: "Classic 1.12.1".to_string(),
                version: "1.0.0".to_string(),
                description: String::new(),
                dependencies: Vec::new(),
            })
            .unwrap();

        database
    }

    fn database_with_aura() -> ContentDatabase {
        let mut database = database_with_pack();

        database.register_aura(aura_definition()).unwrap();

        database
    }

    fn resolved_effect(operation: ResolvedEffectOperation) -> ResolvedEffect {
        ResolvedEffect {
            effect: ContentId::new("classic_1_12_1:effect.test").unwrap(),
            trigger: EffectTrigger::OnCast,
            source: Some(source()),
            target: target(),
            operation,
        }
    }

    fn aura_id() -> ContentId {
        ContentId::new("classic_1_12_1:aura.test").unwrap()
    }

    #[test]
    fn apply_aura_execution_creates_active_aura() {
        let database = database_with_aura();

        let effect = resolved_effect(ResolvedEffectOperation::ApplyAura {
            aura: aura_id(),
            stacks: 2,
            duration_seconds: Some(15.0),
        });

        let mut active_auras = ActiveAuras::default();

        let result = execute_aura_effect(&effect, &database, Some(&mut active_auras)).unwrap();

        assert_eq!(
            result,
            AuraExecutionResult::Applied {
                aura: aura_id(),
                total_stacks: 2,
            }
        );

        let active = active_auras.get(&aura_id()).unwrap();

        assert_eq!(active.stacks(), 2);
        assert_eq!(active.remaining_duration(), Some(15.0));
        assert_eq!(active.source(), Some(source()));
    }

    #[test]
    fn apply_aura_execution_respects_stacking_limits() {
        let database = database_with_aura();

        let effect = resolved_effect(ResolvedEffectOperation::ApplyAura {
            aura: aura_id(),
            stacks: 4,
            duration_seconds: Some(10.0),
        });

        let mut active_auras = ActiveAuras::default();

        execute_aura_effect(&effect, &database, Some(&mut active_auras)).unwrap();

        execute_aura_effect(&effect, &database, Some(&mut active_auras)).unwrap();

        assert_eq!(active_auras.stacks(&aura_id()), 5);
    }

    #[test]
    fn remove_aura_execution_removes_requested_stacks() {
        let definition = aura_definition();
        let database = database_with_aura();

        let mut active_auras = ActiveAuras::default();

        active_auras
            .apply(&definition, Some(source()), 3, Some(20.0))
            .unwrap();

        let effect = resolved_effect(ResolvedEffectOperation::RemoveAura {
            aura: aura_id(),
            stacks: Some(1),
        });

        let result = execute_aura_effect(&effect, &database, Some(&mut active_auras)).unwrap();

        assert_eq!(
            result,
            AuraExecutionResult::Removed {
                aura: aura_id(),
                removed_stacks: 1,
            }
        );

        assert_eq!(active_auras.stacks(&aura_id()), 2);
    }

    #[test]
    fn remove_aura_execution_removes_all_instances() {
        let definition = aura_definition();
        let database = database_with_aura();

        let mut active_auras = ActiveAuras::default();

        active_auras
            .apply(&definition, Some(source()), 2, Some(10.0))
            .unwrap();

        active_auras
            .apply(&definition, Some(source()), 2, Some(20.0))
            .unwrap();

        let effect = resolved_effect(ResolvedEffectOperation::RemoveAura {
            aura: aura_id(),
            stacks: None,
        });

        let result = execute_aura_effect(&effect, &database, Some(&mut active_auras)).unwrap();

        assert_eq!(
            result,
            AuraExecutionResult::Removed {
                aura: aura_id(),
                removed_stacks: 4,
            }
        );

        assert!(active_auras.is_empty());
    }

    #[test]
    fn execution_rejects_unregistered_aura() {
        let database = database_with_pack();

        let effect = resolved_effect(ResolvedEffectOperation::ApplyAura {
            aura: aura_id(),
            stacks: 1,
            duration_seconds: Some(10.0),
        });

        let mut active_auras = ActiveAuras::default();

        assert_eq!(
            execute_aura_effect(&effect, &database, Some(&mut active_auras),),
            Err(AuraExecutionError::MissingAuraDefinition(aura_id()))
        );

        assert!(active_auras.is_empty());
    }

    #[test]
    fn execution_rejects_missing_target_component() {
        let database = database_with_aura();

        let effect = resolved_effect(ResolvedEffectOperation::ApplyAura {
            aura: aura_id(),
            stacks: 1,
            duration_seconds: Some(10.0),
        });

        assert_eq!(
            execute_aura_effect(&effect, &database, None,),
            Err(AuraExecutionError::MissingTargetAuras)
        );
    }

    #[test]
    fn execution_rejects_invalid_applications_without_mutation() {
        let database = database_with_aura();

        let effect = resolved_effect(ResolvedEffectOperation::ApplyAura {
            aura: aura_id(),
            stacks: 0,
            duration_seconds: Some(10.0),
        });

        let mut active_auras = ActiveAuras::default();

        assert_eq!(
            execute_aura_effect(&effect, &database, Some(&mut active_auras),),
            Err(AuraExecutionError::Application(
                AuraRuntimeError::ZeroStacks
            ))
        );

        assert!(active_auras.is_empty());
    }

    #[test]
    fn execution_rejects_non_aura_operations() {
        let database = database_with_aura();

        let effect = resolved_effect(ResolvedEffectOperation::Interrupt);

        let mut active_auras = ActiveAuras::default();

        assert_eq!(
            execute_aura_effect(&effect, &database, Some(&mut active_auras),),
            Err(AuraExecutionError::UnsupportedOperation)
        );

        assert!(active_auras.is_empty());
    }
}
