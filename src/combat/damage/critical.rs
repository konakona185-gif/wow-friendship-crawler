#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CriticalStrikeResult {
    pub incoming_amount: f32,
    pub final_amount: f32,
    pub critical_chance_percent: f32,
    pub critical_multiplier: f32,
    pub roll_percent: f32,
    pub is_critical: bool,
}

impl CriticalStrikeResult {
    pub fn new(
        incoming_amount: f32,
        critical_chance_percent: f32,
        critical_multiplier: f32,
        roll_percent: f32,
    ) -> Result<Self, CriticalStrikeError> {
        if !incoming_amount.is_finite() || incoming_amount < 0.0 {
            return Err(CriticalStrikeError::InvalidIncomingAmount);
        }

        if !critical_chance_percent.is_finite()
            || !(0.0..=100.0).contains(&critical_chance_percent)
        {
            return Err(CriticalStrikeError::InvalidChance);
        }

        if !critical_multiplier.is_finite() || critical_multiplier < 1.0 {
            return Err(CriticalStrikeError::InvalidMultiplier);
        }

        if !roll_percent.is_finite() || !(0.0..100.0).contains(&roll_percent) {
            return Err(CriticalStrikeError::InvalidRoll);
        }

        let is_critical = roll_percent < critical_chance_percent;
        let final_amount = if is_critical {
            incoming_amount * critical_multiplier
        } else {
            incoming_amount
        };

        if !final_amount.is_finite() {
            return Err(CriticalStrikeError::InvalidFinalAmount);
        }

        let result = Self {
            incoming_amount,
            final_amount,
            critical_chance_percent,
            critical_multiplier,
            roll_percent,
            is_critical,
        };

        result.validate()?;

        Ok(result)
    }

    pub fn bonus_amount(&self) -> f32 {
        self.final_amount - self.incoming_amount
    }

    pub fn validate(&self) -> Result<(), CriticalStrikeError> {
        if !self.incoming_amount.is_finite() || self.incoming_amount < 0.0 {
            return Err(CriticalStrikeError::InvalidIncomingAmount);
        }

        if !self.critical_chance_percent.is_finite()
            || !(0.0..=100.0).contains(&self.critical_chance_percent)
        {
            return Err(CriticalStrikeError::InvalidChance);
        }

        if !self.critical_multiplier.is_finite()
            || self.critical_multiplier < 1.0
        {
            return Err(CriticalStrikeError::InvalidMultiplier);
        }

        if !self.roll_percent.is_finite()
            || !(0.0..100.0).contains(&self.roll_percent)
        {
            return Err(CriticalStrikeError::InvalidRoll);
        }

        if !self.final_amount.is_finite() || self.final_amount < 0.0 {
            return Err(CriticalStrikeError::InvalidFinalAmount);
        }

        let expected_critical =
            self.roll_percent < self.critical_chance_percent;

        let expected_amount = if expected_critical {
            self.incoming_amount * self.critical_multiplier
        } else {
            self.incoming_amount
        };

        if !expected_amount.is_finite() {
            return Err(CriticalStrikeError::InvalidFinalAmount);
        }

        if self.is_critical != expected_critical
            || self.final_amount != expected_amount
        {
            return Err(CriticalStrikeError::InconsistentResult);
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CriticalStrikeError {
    InvalidIncomingAmount,
    InvalidChance,
    InvalidMultiplier,
    InvalidRoll,
    InvalidFinalAmount,
    InconsistentResult,
}

impl std::fmt::Display for CriticalStrikeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InvalidIncomingAmount => {
                "Il danno in ingresso deve essere finito e non negativo"
            }
            Self::InvalidChance => {
                "La probabilità di critico deve essere compresa tra 0 e 100"
            }
            Self::InvalidMultiplier => {
                "Il moltiplicatore critico deve essere finito e almeno pari a 1"
            }
            Self::InvalidRoll => {
                "Il tiro critico deve essere compreso tra 0 incluso e 100 escluso"
            }
            Self::InvalidFinalAmount => {
                "Il danno finale del critico deve essere finito e non negativo"
            }
            Self::InconsistentResult => {
                "Il risultato del colpo critico non corrisponde ai parametri forniti"
            }
        };

        formatter.write_str(message)
    }
}

impl std::error::Error for CriticalStrikeError {}

pub fn calculate_critical_strike(
    incoming_amount: f32,
    critical_chance_percent: f32,
    critical_multiplier: f32,
    roll_percent: f32,
) -> Result<CriticalStrikeResult, CriticalStrikeError> {
    CriticalStrikeResult::new(
        incoming_amount,
        critical_chance_percent,
        critical_multiplier,
        roll_percent,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn critical_strike_applies_multiplier_when_roll_is_below_chance() {
        let result =
            calculate_critical_strike(100.0, 25.0, 2.0, 24.99).unwrap();

        assert!(result.is_critical);
        assert_eq!(result.final_amount, 200.0);
        assert_eq!(result.bonus_amount(), 100.0);
        assert!(result.validate().is_ok());
    }

    #[test]
    fn critical_strike_does_not_apply_when_roll_equals_chance() {
        let result =
            calculate_critical_strike(100.0, 25.0, 2.0, 25.0).unwrap();

        assert!(!result.is_critical);
        assert_eq!(result.final_amount, 100.0);
        assert_eq!(result.bonus_amount(), 0.0);
    }

    #[test]
    fn critical_strike_does_not_apply_when_roll_exceeds_chance() {
        let result =
            calculate_critical_strike(100.0, 25.0, 2.0, 75.0).unwrap();

        assert!(!result.is_critical);
        assert_eq!(result.final_amount, 100.0);
    }

    #[test]
    fn critical_strike_with_zero_chance_never_criticals() {
        let result =
            calculate_critical_strike(100.0, 0.0, 2.0, 0.0).unwrap();

        assert!(!result.is_critical);
        assert_eq!(result.final_amount, 100.0);
    }

    #[test]
    fn critical_strike_with_full_chance_always_criticals() {
        let result =
            calculate_critical_strike(100.0, 100.0, 2.0, 99.99).unwrap();

        assert!(result.is_critical);
        assert_eq!(result.final_amount, 200.0);
    }

    #[test]
    fn critical_strike_supports_custom_multipliers() {
        let result =
            calculate_critical_strike(100.0, 100.0, 1.5, 50.0).unwrap();

        assert!(result.is_critical);
        assert_eq!(result.final_amount, 150.0);
        assert_eq!(result.bonus_amount(), 50.0);
    }

    #[test]
    fn critical_strike_preserves_zero_damage() {
        let result =
            calculate_critical_strike(0.0, 100.0, 2.0, 50.0).unwrap();

        assert!(result.is_critical);
        assert_eq!(result.final_amount, 0.0);
        assert_eq!(result.bonus_amount(), 0.0);
    }

    #[test]
    fn critical_strike_rejects_invalid_incoming_amounts() {
        for amount in [-1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert_eq!(
                calculate_critical_strike(amount, 25.0, 2.0, 10.0),
                Err(CriticalStrikeError::InvalidIncomingAmount)
            );
        }
    }

    #[test]
    fn critical_strike_rejects_invalid_chances() {
        for chance in [-1.0, 100.01, f32::NAN, f32::INFINITY] {
            assert_eq!(
                calculate_critical_strike(100.0, chance, 2.0, 10.0),
                Err(CriticalStrikeError::InvalidChance)
            );
        }
    }

    #[test]
    fn critical_strike_rejects_invalid_multipliers() {
        for multiplier in [0.0, 0.5, f32::NAN, f32::INFINITY] {
            assert_eq!(
                calculate_critical_strike(100.0, 25.0, multiplier, 10.0),
                Err(CriticalStrikeError::InvalidMultiplier)
            );
        }
    }

    #[test]
    fn critical_strike_rejects_invalid_rolls() {
        for roll in [-1.0, 100.0, f32::NAN, f32::INFINITY] {
            assert_eq!(
                calculate_critical_strike(100.0, 25.0, 2.0, roll),
                Err(CriticalStrikeError::InvalidRoll)
            );
        }
    }

    #[test]
    fn critical_strike_rejects_final_amount_overflow() {
        assert_eq!(
            calculate_critical_strike(f32::MAX, 100.0, 2.0, 50.0),
            Err(CriticalStrikeError::InvalidFinalAmount)
        );
    }

    #[test]
    fn critical_strike_rejects_inconsistent_results() {
        let mut result =
            calculate_critical_strike(100.0, 25.0, 2.0, 10.0).unwrap();

        result.final_amount = 100.0;

        assert_eq!(
            result.validate(),
            Err(CriticalStrikeError::InconsistentResult)
        );
    }
}
