use std::fmt::{Display, Formatter};

use crate::content::{
    EffectMagnitude,
    EffectValidationError,
};

use crate::combat::CombatAttributes;

pub fn calculate_effect_magnitude(
    magnitude: &EffectMagnitude,
    source_attributes: &CombatAttributes,
) -> Result<f32, EffectMagnitudeError> {
    calculate_effect_magnitude_with_sign(
        magnitude,
        source_attributes,
        false,
    )
}

pub fn calculate_signed_effect_magnitude(
    magnitude: &EffectMagnitude,
    source_attributes: &CombatAttributes,
) -> Result<f32, EffectMagnitudeError> {
    calculate_effect_magnitude_with_sign(
        magnitude,
        source_attributes,
        true,
    )
}

fn calculate_effect_magnitude_with_sign(
    magnitude: &EffectMagnitude,
    source_attributes: &CombatAttributes,
    allow_negative: bool,
) -> Result<f32, EffectMagnitudeError> {
    let validation = if allow_negative {
        magnitude.validate_signed()
    } else {
        magnitude.validate()
    };

    validation.map_err(EffectMagnitudeError::InvalidDefinition)?;

    let mut amount = magnitude.base as f64;

    for scaling in &magnitude.scaling {
        let attribute_value = source_attributes.value(&scaling.stat);

        amount += attribute_value as f64 * scaling.coefficient as f64;
    }

    if !amount.is_finite()
        || (!allow_negative && amount < 0.0)
        || amount.abs() > f32::MAX as f64
    {
        return Err(EffectMagnitudeError::InvalidCalculatedAmount);
    }

    let result = amount as f32;

    if !result.is_finite() {
        return Err(EffectMagnitudeError::InvalidCalculatedAmount);
    }

    Ok(result)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectMagnitudeError {
    InvalidDefinition(EffectValidationError),
    InvalidCalculatedAmount,
}

impl Display for EffectMagnitudeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidDefinition(error) => {
                write!(formatter, "Magnitudine dell'effetto non valida: {error}")
            }
            Self::InvalidCalculatedAmount => {
                formatter.write_str(
                    "Il valore calcolato dell'effetto non è valido",
                )
            }
        }
    }
}

impl std::error::Error for EffectMagnitudeError {}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::content::{
        ContentId,
        EffectScaling,
    };

    fn stat_id(name: &str) -> ContentId {
        ContentId::new(format!("classic_1_12_1:stat.{name}")).unwrap()
    }

    fn magnitude(base: f32) -> EffectMagnitude {
        EffectMagnitude {
            base,
            scaling: Vec::new(),
        }
    }

    fn attributes_with(stat: &str, value: f32) -> CombatAttributes {
        let mut attributes = CombatAttributes::default();
        attributes.set(stat_id(stat), value).unwrap();
        attributes
    }

    #[test]
    fn effect_magnitude_uses_base_without_scaling() {
        let result = calculate_effect_magnitude(
            &magnitude(30.0),
            &CombatAttributes::default(),
        )
        .unwrap();

        assert_eq!(result, 30.0);
    }

    #[test]
    fn effect_magnitude_applies_attribute_scaling() {
        let magnitude = EffectMagnitude {
            base: 10.0,
            scaling: vec![EffectScaling {
                stat: stat_id("spell_power"),
                coefficient: 0.5,
            }],
        };

        let attributes = attributes_with("spell_power", 100.0);

        let result =
            calculate_effect_magnitude(&magnitude, &attributes).unwrap();

        assert_eq!(result, 60.0);
    }

    #[test]
    fn effect_magnitude_adds_multiple_scaling_components() {
        let magnitude = EffectMagnitude {
            base: 10.0,
            scaling: vec![
                EffectScaling {
                    stat: stat_id("spell_power"),
                    coefficient: 0.5,
                },
                EffectScaling {
                    stat: stat_id("attack_power"),
                    coefficient: 0.25,
                },
            ],
        };

        let mut attributes = CombatAttributes::default();
        attributes
            .set(stat_id("spell_power"), 100.0)
            .unwrap();
        attributes
            .set(stat_id("attack_power"), 40.0)
            .unwrap();

        let result =
            calculate_effect_magnitude(&magnitude, &attributes).unwrap();

        assert_eq!(result, 70.0);
    }

    #[test]
    fn effect_magnitude_uses_zero_for_missing_attributes() {
        let magnitude = EffectMagnitude {
            base: 25.0,
            scaling: vec![EffectScaling {
                stat: stat_id("spell_power"),
                coefficient: 0.75,
            }],
        };

        let result = calculate_effect_magnitude(
            &magnitude,
            &CombatAttributes::default(),
        )
        .unwrap();

        assert_eq!(result, 25.0);
    }

    #[test]
    fn effect_magnitude_supports_negative_coefficients() {
        let magnitude = EffectMagnitude {
            base: 100.0,
            scaling: vec![EffectScaling {
                stat: stat_id("spell_power"),
                coefficient: -0.5,
            }],
        };

        let attributes = attributes_with("spell_power", 50.0);

        let result =
            calculate_effect_magnitude(&magnitude, &attributes).unwrap();

        assert_eq!(result, 75.0);
    }

    #[test]
    fn effect_magnitude_rejects_negative_base_values() {
        let result = calculate_effect_magnitude(
            &magnitude(-10.0),
            &CombatAttributes::default(),
        );

        assert_eq!(
            result,
            Err(EffectMagnitudeError::InvalidDefinition(
                EffectValidationError::InvalidMagnitude
            ))
        );
    }

    #[test]
    fn effect_magnitude_rejects_non_finite_coefficients() {
        let magnitude = EffectMagnitude {
            base: 50.0,
            scaling: vec![EffectScaling {
                stat: stat_id("spell_power"),
                coefficient: f32::INFINITY,
            }],
        };

        let result = calculate_effect_magnitude(
            &magnitude,
            &CombatAttributes::default(),
        );

        assert_eq!(
            result,
            Err(EffectMagnitudeError::InvalidDefinition(
                EffectValidationError::InvalidScalingCoefficient
            ))
        );
    }

    #[test]
    fn effect_magnitude_rejects_negative_calculated_amounts() {
        let magnitude = EffectMagnitude {
            base: 10.0,
            scaling: vec![EffectScaling {
                stat: stat_id("spell_power"),
                coefficient: -1.0,
            }],
        };

        let attributes = attributes_with("spell_power", 20.0);

        assert_eq!(
            calculate_effect_magnitude(&magnitude, &attributes),
            Err(EffectMagnitudeError::InvalidCalculatedAmount)
        );
    }

    #[test]
    fn effect_magnitude_rejects_values_above_f32_range() {
        let magnitude = EffectMagnitude {
            base: f32::MAX,
            scaling: vec![EffectScaling {
                stat: stat_id("spell_power"),
                coefficient: 2.0,
            }],
        };

        let attributes = attributes_with("spell_power", f32::MAX);

        assert_eq!(
            calculate_effect_magnitude(&magnitude, &attributes),
            Err(EffectMagnitudeError::InvalidCalculatedAmount)
        );
    }

    #[test]
    fn signed_effect_magnitude_accepts_negative_base_values() {
        let result = calculate_signed_effect_magnitude(
            &magnitude(-10.0),
            &CombatAttributes::default(),
        )
        .unwrap();

        assert_eq!(result, -10.0);
    }

    #[test]
    fn signed_effect_magnitude_allows_negative_calculated_amounts() {
        let magnitude = EffectMagnitude {
            base: 10.0,
            scaling: vec![EffectScaling {
                stat: stat_id("spell_power"),
                coefficient: -1.0,
            }],
        };

        let attributes = attributes_with("spell_power", 20.0);

        let result =
            calculate_signed_effect_magnitude(&magnitude, &attributes)
                .unwrap();

        assert_eq!(result, -10.0);
    }

    #[test]
    fn signed_effect_magnitude_applies_positive_scaling() {
        let magnitude = EffectMagnitude {
            base: -10.0,
            scaling: vec![EffectScaling {
                stat: stat_id("spell_power"),
                coefficient: 0.5,
            }],
        };

        let attributes = attributes_with("spell_power", 40.0);

        let result =
            calculate_signed_effect_magnitude(&magnitude, &attributes)
                .unwrap();

        assert_eq!(result, 10.0);
    }

    #[test]
    fn signed_effect_magnitude_rejects_non_finite_coefficients() {
        let magnitude = EffectMagnitude {
            base: -10.0,
            scaling: vec![EffectScaling {
                stat: stat_id("spell_power"),
                coefficient: f32::NAN,
            }],
        };

        let result = calculate_signed_effect_magnitude(
            &magnitude,
            &CombatAttributes::default(),
        );

        assert_eq!(
            result,
            Err(EffectMagnitudeError::InvalidDefinition(
                EffectValidationError::InvalidScalingCoefficient
            ))
        );
    }

    #[test]
    fn signed_effect_magnitude_rejects_values_outside_f32_range() {
        let magnitude = EffectMagnitude {
            base: f32::MAX,
            scaling: vec![EffectScaling {
                stat: stat_id("spell_power"),
               coefficient: -3.0,
            }],
        };

        let attributes = attributes_with("spell_power", f32::MAX);

        assert_eq!(
            calculate_signed_effect_magnitude(&magnitude, &attributes),
            Err(EffectMagnitudeError::InvalidCalculatedAmount)
        );
    }
}
