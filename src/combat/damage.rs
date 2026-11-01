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
}
