use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

use super::{ContentId, ContentMetadata, ContentMetadataError};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EffectDefinition {
    pub metadata: ContentMetadata,
    pub trigger: EffectTrigger,
    pub target: EffectTarget,
    pub kind: EffectKind,
}

impl EffectDefinition {
    pub fn validate(&self) -> Result<(), EffectValidationError> {
        self.metadata
            .validate()
            .map_err(EffectValidationError::InvalidMetadata)?;

        match &self.target {
            EffectTarget::AlliesInRadius { radius }
            | EffectTarget::EnemiesInRadius { radius }
            | EffectTarget::UnitsInRadius { radius } => {
                if !radius.is_finite() || *radius <= 0.0 {
                    return Err(EffectValidationError::InvalidRadius);
                }
            }
            EffectTarget::Caster | EffectTarget::SelectedTarget => {}
        }

        match &self.kind {
            EffectKind::Damage { amount, .. }
            | EffectKind::Heal { amount }
            | EffectKind::GainResource { amount, .. }
            | EffectKind::SpendResource { amount, .. }
            | EffectKind::ModifyThreat { amount } => {
                amount.validate()?;
            }
            EffectKind::ApplyAura {
                stacks,
                duration_seconds,
                ..
            } => {
                if *stacks == 0 {
                    return Err(EffectValidationError::ZeroStacks);
                }

                validate_duration(*duration_seconds)?;
            }
            EffectKind::RemoveAura { stacks, .. } => {
                if stacks.is_some_and(|value| value == 0) {
                    return Err(EffectValidationError::ZeroStacks);
                }
            }
            EffectKind::Summon {
                count,
                duration_seconds,
                ..
            } => {
                if *count == 0 {
                    return Err(EffectValidationError::ZeroSummonCount);
                }

                validate_duration(*duration_seconds)?;
            }
            EffectKind::Displacement { distance } => {
                if !distance.is_finite() || *distance <= 0.0 {
                    return Err(EffectValidationError::InvalidDisplacement);
                }
            }
            EffectKind::Interrupt | EffectKind::Custom { .. } => {}
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectTrigger {
    OnCast,
    OnHit,
    OnCriticalHit,
    OnPeriodicTick,
    OnKill,
    OnDamageTaken,
    OnDamageDealt,
    OnDodge,
    OnBlock,
    OnExpire,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "parameters", rename_all = "snake_case")]
pub enum EffectTarget {
    Caster,
    SelectedTarget,
    AlliesInRadius { radius: f32 },
    EnemiesInRadius { radius: f32 },
    UnitsInRadius { radius: f32 },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EffectMagnitude {
    pub base: f32,
    #[serde(default)]
    pub scaling: Vec<EffectScaling>,
}

impl EffectMagnitude {
    pub fn validate(&self) -> Result<(), EffectValidationError> {
        if !self.base.is_finite() || self.base < 0.0 {
            return Err(EffectValidationError::InvalidMagnitude);
        }

        for scaling in &self.scaling {
            if !scaling.coefficient.is_finite() {
                return Err(EffectValidationError::InvalidScalingCoefficient);
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EffectScaling {
    pub stat: ContentId,
    pub coefficient: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "parameters", rename_all = "snake_case")]
pub enum EffectKind {
    Damage {
        amount: EffectMagnitude,
        school: ContentId,
    },
    Heal {
        amount: EffectMagnitude,
    },
    GainResource {
        resource: ContentId,
        amount: EffectMagnitude,
    },
    SpendResource {
        resource: ContentId,
        amount: EffectMagnitude,
    },
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
        amount: EffectMagnitude,
    },
    Interrupt,
    Displacement {
        distance: f32,
    },
    Custom {
        handler: ContentId,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectValidationError {
    InvalidMetadata(ContentMetadataError),
    InvalidMagnitude,
    InvalidScalingCoefficient,
    InvalidRadius,
    InvalidDuration,
    ZeroStacks,
    ZeroSummonCount,
    InvalidDisplacement,
}

impl Display for EffectValidationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InvalidMetadata(error) => {
                return write!(formatter, "Metadati dell'effetto non validi: {error}");
            }
            Self::InvalidMagnitude => {
                "Il valore dell'effetto deve essere finito e non negativo"
            }
            Self::InvalidScalingCoefficient => {
                "Il coefficiente di scaling deve essere finito"
            }
            Self::InvalidRadius => {
                "Il raggio dell'effetto deve essere finito e positivo"
            }
            Self::InvalidDuration => {
                "La durata deve essere finita e positiva"
            }
            Self::ZeroStacks => {
                "Il numero di stack deve essere maggiore di zero"
            }
            Self::ZeroSummonCount => {
                "Il numero di creature evocate deve essere maggiore di zero"
            }
            Self::InvalidDisplacement => {
                "La distanza di spostamento deve essere finita e positiva"
            }
        };

        formatter.write_str(message)
    }
}

impl std::error::Error for EffectValidationError {}

fn validate_duration(
    duration: Option<f32>,
) -> Result<(), EffectValidationError> {
    if duration.is_some_and(|value| !value.is_finite() || value <= 0.0) {
        return Err(EffectValidationError::InvalidDuration);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_metadata() -> ContentMetadata {
        ContentMetadata {
            id: ContentId::new("classic_1_12_1:effect.test").unwrap(),
            pack_id: ContentId::new("pack:classic_1_12_1").unwrap(),
            name: "Test Effect".to_string(),
            description: String::new(),
            icon: None,
            tags: Vec::new(),
        }
    }

    fn valid_damage_effect() -> EffectDefinition {
        EffectDefinition {
            metadata: valid_metadata(),
            trigger: EffectTrigger::OnCast,
            target: EffectTarget::SelectedTarget,
            kind: EffectKind::Damage {
                amount: EffectMagnitude {
                    base: 100.0,
                    scaling: Vec::new(),
                },
                school: ContentId::new(
                    "classic_1_12_1:school.physical",
                )
                .unwrap(),
            },
        }
    }

    #[test]
    fn valid_effect_passes_validation() {
        assert!(valid_damage_effect().validate().is_ok());
    }

    #[test]
    fn effect_rejects_invalid_metadata() {
        let mut effect = valid_damage_effect();
        effect.metadata.name.clear();

        assert_eq!(
            effect.validate(),
            Err(EffectValidationError::InvalidMetadata(
                ContentMetadataError::EmptyName
            ))
        );
    }

    #[test]
    fn effect_rejects_invalid_magnitude() {
        let mut effect = valid_damage_effect();

        effect.kind = EffectKind::Damage {
            amount: EffectMagnitude {
                base: -10.0,
                scaling: Vec::new(),
            },
            school: ContentId::new(
                "classic_1_12_1:school.physical",
            )
            .unwrap(),
        };

        assert_eq!(
            effect.validate(),
            Err(EffectValidationError::InvalidMagnitude)
        );
    }

    #[test]
    fn effect_rejects_invalid_scaling() {
        let mut effect = valid_damage_effect();

        effect.kind = EffectKind::Heal {
            amount: EffectMagnitude {
                base: 50.0,
                scaling: vec![EffectScaling {
                    stat: ContentId::new(
                        "classic_1_12_1:stat.spell_power",
                    )
                    .unwrap(),
                    coefficient: f32::INFINITY,
                }],
            },
        };

        assert_eq!(
            effect.validate(),
            Err(EffectValidationError::InvalidScalingCoefficient)
        );
    }

    #[test]
    fn effect_rejects_invalid_area_radius() {
        let mut effect = valid_damage_effect();
        effect.target = EffectTarget::EnemiesInRadius {
            radius: 0.0,
        };

        assert_eq!(
            effect.validate(),
            Err(EffectValidationError::InvalidRadius)
        );
    }

    #[test]
    fn effect_rejects_invalid_aura_duration_and_stacks() {
        let mut effect = valid_damage_effect();

        effect.kind = EffectKind::ApplyAura {
            aura: ContentId::new(
                "classic_1_12_1:aura.test",
            )
            .unwrap(),
            stacks: 1,
            duration_seconds: Some(-1.0),
        };

        assert_eq!(
            effect.validate(),
            Err(EffectValidationError::InvalidDuration)
        );

        let mut effect = valid_damage_effect();

        effect.kind = EffectKind::ApplyAura {
            aura: ContentId::new(
                "classic_1_12_1:aura.test",
            )
            .unwrap(),
            stacks: 0,
            duration_seconds: Some(5.0),
        };

        assert_eq!(
            effect.validate(),
            Err(EffectValidationError::ZeroStacks)
        );
    }

    #[test]
    fn effect_supports_ron_serialization() {
        let original = valid_damage_effect();

        let serialized = ron::to_string(&original).unwrap();
        let restored: EffectDefinition =
            ron::from_str(&serialized).unwrap();

        assert_eq!(original, restored);
        assert!(restored.validate().is_ok());
    }
}
