use bevy::prelude::Entity;

#[derive(Clone, Debug, PartialEq)]
pub struct HealingRequest {
    pub source: Option<Entity>,
    pub target: Entity,
    pub base_amount: f32,
}

impl HealingRequest {
    pub fn validate(&self) -> Result<(), HealingRequestError> {
        if !self.base_amount.is_finite() || self.base_amount < 0.0 {
            return Err(HealingRequestError::InvalidAmount);
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HealingRequestError {
    InvalidAmount,
}

impl std::fmt::Display for HealingRequestError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InvalidAmount => "La quantità di guarigione deve essere finita e non negativa",
        };

        formatter.write_str(message)
    }
}

impl std::error::Error for HealingRequestError {}

#[derive(Clone, Debug, PartialEq)]
pub struct HealingResolution {
    pub request: HealingRequest,
    pub effective_amount: f32,
}

impl HealingResolution {
    pub fn new(
        request: HealingRequest,
        effective_amount: f32,
    ) -> Result<Self, HealingResolutionError> {
        let resolution = Self {
            request,
            effective_amount,
        };

        resolution.validate()?;

        Ok(resolution)
    }

    pub fn validate(&self) -> Result<(), HealingResolutionError> {
        self.request
            .validate()
            .map_err(HealingResolutionError::InvalidRequest)?;

        if !self.effective_amount.is_finite() || self.effective_amount < 0.0 {
            return Err(HealingResolutionError::InvalidEffectiveAmount);
        }

        if self.effective_amount > self.request.base_amount {
            return Err(HealingResolutionError::EffectiveExceedsRequestedAmount);
        }

        Ok(())
    }

    pub fn overheal_amount(&self) -> f32 {
        self.request.base_amount - self.effective_amount
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HealingResolutionError {
    InvalidRequest(HealingRequestError),
    InvalidEffectiveAmount,
    EffectiveExceedsRequestedAmount,
}

impl std::fmt::Display for HealingResolutionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRequest(error) => {
                write!(formatter, "Richiesta di guarigione non valida: {error}")
            }
            Self::InvalidEffectiveAmount => {
                formatter.write_str("La guarigione effettiva deve essere finita e non negativa")
            }
            Self::EffectiveExceedsRequestedAmount => {
                formatter.write_str("La guarigione effettiva non può superare quella richiesta")
            }
        }
    }
}

impl std::error::Error for HealingResolutionError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_request() -> HealingRequest {
        HealingRequest {
            source: None,
            target: Entity::from_bits(1),
            base_amount: 100.0,
        }
    }

    #[test]
    fn valid_healing_request_passes_validation() {
        assert!(valid_request().validate().is_ok());
    }

    #[test]
    fn healing_request_rejects_invalid_amounts() {
        for amount in [-1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let mut request = valid_request();
            request.base_amount = amount;

            assert_eq!(request.validate(), Err(HealingRequestError::InvalidAmount));
        }
    }

    #[test]
    fn healing_request_accepts_zero_amount() {
        let mut request = valid_request();
        request.base_amount = 0.0;

        assert!(request.validate().is_ok());
    }

    #[test]
    fn healing_resolution_supports_effective_healing() {
        let resolution = HealingResolution::new(valid_request(), 75.0).unwrap();

        assert_eq!(resolution.effective_amount, 75.0);
        assert_eq!(resolution.overheal_amount(), 25.0);
        assert!(resolution.validate().is_ok());
    }

    #[test]
    fn healing_resolution_supports_full_healing() {
        let resolution = HealingResolution::new(valid_request(), 100.0).unwrap();

        assert_eq!(resolution.overheal_amount(), 0.0);
    }

    #[test]
    fn healing_resolution_supports_zero_effective_healing() {
        let resolution = HealingResolution::new(valid_request(), 0.0).unwrap();

        assert_eq!(resolution.overheal_amount(), 100.0);
    }

    #[test]
    fn healing_resolution_rejects_invalid_effective_amounts() {
        for amount in [-1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert_eq!(
                HealingResolution::new(valid_request(), amount),
                Err(HealingResolutionError::InvalidEffectiveAmount)
            );
        }
    }

    #[test]
    fn healing_resolution_rejects_effective_healing_above_requested_amount() {
        assert_eq!(
            HealingResolution::new(valid_request(), 125.0),
            Err(HealingResolutionError::EffectiveExceedsRequestedAmount)
        );
    }

    #[test]
    fn healing_resolution_rejects_invalid_requests() {
        let mut request = valid_request();
        request.base_amount = -10.0;

        assert_eq!(
            HealingResolution::new(request, 0.0),
            Err(HealingResolutionError::InvalidRequest(
                HealingRequestError::InvalidAmount
            ))
        );
    }
}
