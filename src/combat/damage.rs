use bevy::prelude::Entity;

use crate::content::ContentId;

#[derive(Clone, Debug, PartialEq)]
pub struct DamageRequest {
    pub source: Option<Entity>,
    pub target: Entity,
    pub base_amount: f32,
    pub school: ContentId,
}

impl DamageRequest {
    pub fn validate(&self) -> Result<(), DamageRequestError> {
        if !self.base_amount.is_finite() || self.base_amount < 0.0 {
            return Err(DamageRequestError::InvalidAmount);
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DamageRequestError {
    InvalidAmount,
}

impl std::fmt::Display for DamageRequestError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InvalidAmount => {
                "La quantità di danno deve essere finita e non negativa"
            }
        };

        formatter.write_str(message)
    }
}

impl std::error::Error for DamageRequestError {}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DamageMitigationResult {
    pub incoming_amount: f32,
    pub mitigated_amount: f32,
}

impl DamageMitigationResult {
    pub fn new(
        incoming_amount: f32,
        mitigated_amount: f32,
    ) -> Result<Self, DamageMitigationError> {
        let result = Self {
            incoming_amount,
            mitigated_amount,
        };

        result.validate()?;

        Ok(result)
    }

    pub fn validate(&self) -> Result<(), DamageMitigationError> {
        if !self.incoming_amount.is_finite() || self.incoming_amount < 0.0 {
            return Err(DamageMitigationError::InvalidIncomingAmount);
        }

        if !self.mitigated_amount.is_finite() || self.mitigated_amount < 0.0 {
            return Err(DamageMitigationError::InvalidMitigatedAmount);
        }

        if self.mitigated_amount > self.incoming_amount {
            return Err(DamageMitigationError::MitigationExceedsIncomingAmount);
        }

        Ok(())
    }

    pub fn remaining_amount(&self) -> f32 {
        self.incoming_amount - self.mitigated_amount
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DamageMitigationError {
    InvalidIncomingAmount,
    InvalidMitigatedAmount,
    MitigationExceedsIncomingAmount,
}

impl std::fmt::Display for DamageMitigationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InvalidIncomingAmount => {
                "Il danno in ingresso deve essere finito e non negativo"
            }
            Self::InvalidMitigatedAmount => {
                "Il danno mitigato deve essere finito e non negativo"
            }
            Self::MitigationExceedsIncomingAmount => {
                "Il danno mitigato non può superare quello in ingresso"
            }
        };

        formatter.write_str(message)
    }
}

impl std::error::Error for DamageMitigationError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_request() -> DamageRequest {
        DamageRequest {
            source: None,
            target: Entity::from_bits(1),
            base_amount: 100.0,
            school: ContentId::new("classic_1_12_1:school.physical").unwrap(),
        }
    }

    #[test]
    fn valid_damage_request_passes_validation() {
        assert!(valid_request().validate().is_ok());
    }

    #[test]
    fn damage_request_rejects_invalid_amounts() {
        for amount in [-1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let mut request = valid_request();
            request.base_amount = amount;

            assert_eq!(
                request.validate(),
                Err(DamageRequestError::InvalidAmount)
            );
        }
    }

    #[test]
    fn damage_request_accepts_zero_amount() {
        let mut request = valid_request();
        request.base_amount = 0.0;

        assert!(request.validate().is_ok());
    }

    #[test]
    fn mitigation_result_supports_partial_mitigation() {
        let result = DamageMitigationResult::new(100.0, 25.0).unwrap();

        assert_eq!(result.remaining_amount(), 75.0);
    }

    #[test]
    fn mitigation_result_supports_zero_mitigation() {
        let result = DamageMitigationResult::new(100.0, 0.0).unwrap();

        assert_eq!(result.remaining_amount(), 100.0);
    }

    #[test]
    fn mitigation_result_supports_full_mitigation() {
        let result = DamageMitigationResult::new(100.0, 100.0).unwrap();

        assert_eq!(result.remaining_amount(), 0.0);
    }

    #[test]
    fn mitigation_result_rejects_invalid_incoming_amounts() {
        for amount in [-1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert_eq!(
                DamageMitigationResult::new(amount, 0.0),
                Err(DamageMitigationError::InvalidIncomingAmount)
            );
        }
    }

    #[test]
    fn mitigation_result_rejects_invalid_mitigated_amounts() {
        for amount in [-1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert_eq!(
                DamageMitigationResult::new(100.0, amount),
                Err(DamageMitigationError::InvalidMitigatedAmount)
            );
        }
    }

    #[test]
    fn mitigation_result_rejects_mitigation_above_incoming_damage() {
        assert_eq!(
            DamageMitigationResult::new(50.0, 75.0),
            Err(DamageMitigationError::MitigationExceedsIncomingAmount)
        );
    }
}
