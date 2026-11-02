use bevy::prelude::Component;
use std::fmt::{Display, Formatter};

use super::{
    DamageResolution,
    DamageResolutionError,
    HealingResolution,
    HealingResolutionError,
};

#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct HealthPool {
    current: f32,
    maximum: f32,
}

impl HealthPool {
    pub fn new(
        current: f32,
        maximum: f32,
    ) -> Result<Self, HealthPoolError> {
        let pool = Self {
            current,
            maximum,
        };

        pool.validate()?;

        Ok(pool)
    }

    pub fn current(&self) -> f32 {
        self.current
    }

    pub fn maximum(&self) -> f32 {
        self.maximum
    }

    pub fn remaining_capacity(&self) -> f32 {
        self.maximum - self.current
    }

    pub fn set_current(
        &mut self,
        current: f32,
    ) -> Result<(), HealthPoolError> {
        if !current.is_finite() || current < 0.0 {
            return Err(HealthPoolError::InvalidCurrent);
        }

        if current > self.maximum {
            return Err(HealthPoolError::CurrentExceedsMaximum);
        }

        self.current = current;

        Ok(())
    }

    pub fn set_maximum(
        &mut self,
        maximum: f32,
    ) -> Result<(), HealthPoolError> {
        if !maximum.is_finite() || maximum < 0.0 {
            return Err(HealthPoolError::InvalidMaximum);
        }

        if self.current > maximum {
            return Err(HealthPoolError::CurrentExceedsMaximum);
        }

        self.maximum = maximum;

        Ok(())
    }

    pub fn apply_damage(
        &mut self,
        amount: f32,
    ) -> Result<f32, HealthPoolError> {
        if !amount.is_finite() || amount < 0.0 {
            return Err(HealthPoolError::InvalidAmount);
        }

        let effective_amount = self.current.min(amount);
        self.current -= effective_amount;

        Ok(effective_amount)
    }

    pub fn apply_healing(
        &mut self,
        amount: f32,
    ) -> Result<f32, HealthPoolError> {
        if !amount.is_finite() || amount < 0.0 {
            return Err(HealthPoolError::InvalidAmount);
        }

        let effective_amount = self.remaining_capacity().min(amount);
        self.current += effective_amount;

        Ok(effective_amount)
    }

    pub fn apply_damage_resolution(
        &mut self,
        resolution: &DamageResolution,
    ) -> Result<f32, HealthPoolError> {
        resolution
            .validate()
            .map_err(HealthPoolError::InvalidDamageResolution)?;

        self.apply_damage(resolution.final_amount())
    }

    pub fn apply_healing_resolution(
        &mut self,
        resolution: &HealingResolution,
    ) -> Result<f32, HealthPoolError> {
        resolution
            .validate()
            .map_err(HealthPoolError::InvalidHealingResolution)?;

        self.apply_healing(resolution.effective_amount)
    }

    pub fn validate(&self) -> Result<(), HealthPoolError> {
        if !self.current.is_finite() || self.current < 0.0 {
            return Err(HealthPoolError::InvalidCurrent);
        }

        if !self.maximum.is_finite() || self.maximum < 0.0 {
            return Err(HealthPoolError::InvalidMaximum);
        }

        if self.current > self.maximum {
            return Err(HealthPoolError::CurrentExceedsMaximum);
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HealthPoolError {
    InvalidCurrent,
    InvalidMaximum,
    CurrentExceedsMaximum,
    InvalidAmount,
    InvalidDamageResolution(DamageResolutionError),
    InvalidHealingResolution(HealingResolutionError),
}

impl Display for HealthPoolError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidCurrent => {
                formatter.write_str(
                    "La salute corrente deve essere finita e non negativa",
                )
            }
            Self::InvalidMaximum => {
                formatter.write_str(
                    "La salute massima deve essere finita e non negativa",
                )
            }
            Self::CurrentExceedsMaximum => {
                formatter.write_str(
                    "La salute corrente non può superare quella massima",
                )
            }
            Self::InvalidAmount => {
                formatter.write_str(
                    "La quantità deve essere finita e non negativa",
                )
            }
            Self::InvalidDamageResolution(error) => {
                write!(formatter, "Risoluzione del danno non valida: {error}")
            }
            Self::InvalidHealingResolution(error) => {
                write!(formatter, "Risoluzione della guarigione non valida: {error}")
            }
        }
    }
}

impl std::error::Error for HealthPoolError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::{
        DamageMitigationError,
        DamageMitigationResult,
        DamageRequest,
        HealingRequest,
    };
    use crate::content::ContentId;

    fn valid_damage_resolution() -> DamageResolution {
        let request = DamageRequest {
            source: None,
            target: bevy::prelude::Entity::from_bits(1),
            base_amount: 100.0,
            school: ContentId::new(
                "classic_1_12_1:school.physical",
            )
            .unwrap(),
        };

        DamageResolution::new(
            request,
            DamageMitigationResult::new(100.0, 25.0).unwrap(),
        )
        .unwrap()
    }

    fn valid_healing_resolution() -> HealingResolution {
        let request = HealingRequest {
            source: None,
            target: bevy::prelude::Entity::from_bits(1),
            base_amount: 100.0,
        };

        HealingResolution::new(request, 75.0).unwrap()
    }

    #[test]
    fn health_pool_accepts_valid_values() {
        let pool = HealthPool::new(75.0, 100.0).unwrap();

        assert_eq!(pool.current(), 75.0);
        assert_eq!(pool.maximum(), 100.0);
        assert_eq!(pool.remaining_capacity(), 25.0);
    }

    #[test]
    fn health_pool_accepts_zero_values() {
        let pool = HealthPool::new(0.0, 0.0).unwrap();

        assert_eq!(pool.current(), 0.0);
        assert_eq!(pool.maximum(), 0.0);
    }

    #[test]
    fn health_pool_rejects_invalid_current_values() {
        for value in [-1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert_eq!(
                HealthPool::new(value, 100.0),
                Err(HealthPoolError::InvalidCurrent)
            );
        }
    }

    #[test]
    fn health_pool_rejects_invalid_maximum_values() {
        for value in [-1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert_eq!(
                HealthPool::new(50.0, value),
                Err(HealthPoolError::InvalidMaximum)
            );
        }
    }

    #[test]
    fn health_pool_rejects_current_above_maximum() {
        assert_eq!(
            HealthPool::new(150.0, 100.0),
            Err(HealthPoolError::CurrentExceedsMaximum)
        );
    }

    #[test]
    fn health_pool_can_update_current_value() {
        let mut pool = HealthPool::new(50.0, 100.0).unwrap();

        pool.set_current(75.0).unwrap();

        assert_eq!(pool.current(), 75.0);
        assert_eq!(pool.remaining_capacity(), 25.0);
    }

    #[test]
    fn health_pool_rejects_invalid_current_updates() {
        let mut pool = HealthPool::new(50.0, 100.0).unwrap();

        assert_eq!(
            pool.set_current(150.0),
            Err(HealthPoolError::CurrentExceedsMaximum)
        );

        assert_eq!(pool.current(), 50.0);
    }

    #[test]
    fn health_pool_can_update_maximum_value() {
        let mut pool = HealthPool::new(50.0, 100.0).unwrap();

        pool.set_maximum(150.0).unwrap();

        assert_eq!(pool.maximum(), 150.0);
        assert_eq!(pool.remaining_capacity(), 100.0);
    }

    #[test]
    fn health_pool_rejects_maximum_below_current() {
        let mut pool = HealthPool::new(50.0, 100.0).unwrap();

        assert_eq!(
            pool.set_maximum(25.0),
            Err(HealthPoolError::CurrentExceedsMaximum)
        );

        assert_eq!(pool.maximum(), 100.0);
    }

    #[test]
    fn health_pool_applies_partial_damage() {
        let mut pool = HealthPool::new(100.0, 100.0).unwrap();

        let effective = pool.apply_damage(30.0).unwrap();

        assert_eq!(effective, 30.0);
        assert_eq!(pool.current(), 70.0);
    }

    #[test]
    fn health_pool_caps_damage_at_current_health() {
        let mut pool = HealthPool::new(40.0, 100.0).unwrap();

        let effective = pool.apply_damage(100.0).unwrap();

        assert_eq!(effective, 40.0);
        assert_eq!(pool.current(), 0.0);
    }

    #[test]
    fn health_pool_applies_partial_healing() {
        let mut pool = HealthPool::new(50.0, 100.0).unwrap();

        let effective = pool.apply_healing(30.0).unwrap();

        assert_eq!(effective, 30.0);
        assert_eq!(pool.current(), 80.0);
    }

    #[test]
    fn health_pool_caps_healing_at_remaining_capacity() {
        let mut pool = HealthPool::new(80.0, 100.0).unwrap();

        let effective = pool.apply_healing(50.0).unwrap();

        assert_eq!(effective, 20.0);
        assert_eq!(pool.current(), 100.0);
    }

    #[test]
    fn health_pool_rejects_invalid_damage_and_healing() {
        for value in [-1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let mut pool = HealthPool::new(50.0, 100.0).unwrap();

            assert_eq!(
                pool.apply_damage(value),
                Err(HealthPoolError::InvalidAmount)
            );

            assert_eq!(
                pool.apply_healing(value),
                Err(HealthPoolError::InvalidAmount)
            );

            assert_eq!(pool.current(), 50.0);
        }
    }

    #[test]
    fn health_pool_applies_valid_damage_resolutions() {
        let mut pool = HealthPool::new(100.0, 100.0).unwrap();
        let resolution = valid_damage_resolution();

        let effective = pool.apply_damage_resolution(&resolution).unwrap();

        assert_eq!(effective, 75.0);
        assert_eq!(pool.current(), 25.0);
    }

    #[test]
    fn health_pool_caps_damage_resolution_at_current_health() {
        let mut pool = HealthPool::new(40.0, 100.0).unwrap();
        let resolution = valid_damage_resolution();

        let effective = pool.apply_damage_resolution(&resolution).unwrap();

        assert_eq!(effective, 40.0);
        assert_eq!(pool.current(), 0.0);
    }

    #[test]
    fn health_pool_rejects_invalid_damage_resolutions_without_mutation() {
        let mut pool = HealthPool::new(100.0, 100.0).unwrap();
        let mut resolution = valid_damage_resolution();
        resolution.mitigation.mitigated_amount = 150.0;

        assert_eq!(
            pool.apply_damage_resolution(&resolution),
            Err(HealthPoolError::InvalidDamageResolution(
                DamageResolutionError::InvalidMitigation(
                    DamageMitigationError::MitigationExceedsIncomingAmount
                )
            ))
        );

        assert_eq!(pool.current(), 100.0);
    }

    #[test]
    fn health_pool_applies_valid_healing_resolutions() {
        let mut pool = HealthPool::new(50.0, 100.0).unwrap();
        let resolution = valid_healing_resolution();

        let effective = pool.apply_healing_resolution(&resolution).unwrap();

        assert_eq!(effective, 50.0);
        assert_eq!(pool.current(), 100.0);
    }

    #[test]
    fn health_pool_rejects_invalid_healing_resolutions_without_mutation() {
        let mut pool = HealthPool::new(50.0, 100.0).unwrap();
        let mut resolution = valid_healing_resolution();
        resolution.effective_amount = 150.0;

        assert_eq!(
            pool.apply_healing_resolution(&resolution),
            Err(HealthPoolError::InvalidHealingResolution(
                HealingResolutionError::EffectiveExceedsRequestedAmount
            ))
        );

        assert_eq!(pool.current(), 50.0);
    }
}
