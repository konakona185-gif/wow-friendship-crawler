use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use bevy::prelude::{Component, Entity};

use crate::content::ContentId;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResourcePool {
    current: f32,
    maximum: f32,
}

impl ResourcePool {
    pub fn new(
        current: f32,
        maximum: f32,
    ) -> Result<Self, ResourcePoolError> {
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
    ) -> Result<(), ResourcePoolError> {
        if !current.is_finite() || current < 0.0 {
            return Err(ResourcePoolError::InvalidCurrent);
        }

        if current > self.maximum {
            return Err(ResourcePoolError::CurrentExceedsMaximum);
        }

        self.current = current;

        Ok(())
    }

    pub fn set_maximum(
        &mut self,
        maximum: f32,
    ) -> Result<(), ResourcePoolError> {
        if !maximum.is_finite() || maximum < 0.0 {
            return Err(ResourcePoolError::InvalidMaximum);
        }

        if self.current > maximum {
            return Err(ResourcePoolError::CurrentExceedsMaximum);
        }

        self.maximum = maximum;

        Ok(())
    }

    pub fn validate(&self) -> Result<(), ResourcePoolError> {
        if !self.current.is_finite() || self.current < 0.0 {
            return Err(ResourcePoolError::InvalidCurrent);
        }

        if !self.maximum.is_finite() || self.maximum < 0.0 {
            return Err(ResourcePoolError::InvalidMaximum);
        }

        if self.current > self.maximum {
            return Err(ResourcePoolError::CurrentExceedsMaximum);
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourcePoolError {
    InvalidCurrent,
    InvalidMaximum,
    CurrentExceedsMaximum,
}

impl Display for ResourcePoolError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InvalidCurrent => {
                "Il valore corrente della risorsa deve essere finito e non negativo"
            }
            Self::InvalidMaximum => {
                "Il valore massimo della risorsa deve essere finito e non negativo"
            }
            Self::CurrentExceedsMaximum => {
                "Il valore corrente non può superare quello massimo"
            }
        };

        formatter.write_str(message)
    }
}

impl std::error::Error for ResourcePoolError {}

#[derive(Component, Clone, Debug, Default)]
pub struct CombatResources {
    pools: HashMap<ContentId, ResourcePool>,
}

impl CombatResources {
    pub fn pool(&self, resource: &ContentId) -> Option<&ResourcePool> {
        self.pools.get(resource)
    }

    pub fn pool_mut(
        &mut self,
        resource: &ContentId,
    ) -> Option<&mut ResourcePool> {
        self.pools.get_mut(resource)
    }

    pub fn insert(
        &mut self,
        resource: ContentId,
        pool: ResourcePool,
    ) -> Option<ResourcePool> {
        self.pools.insert(resource, pool)
    }

    pub fn remove(&mut self, resource: &ContentId) -> Option<ResourcePool> {
        self.pools.remove(resource)
    }

    pub fn len(&self) -> usize {
        self.pools.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pools.is_empty()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceOperation {
    Gain,
    Spend,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResourceRequest {
    pub source: Option<Entity>,
    pub target: Entity,
    pub resource: ContentId,
    pub operation: ResourceOperation,
    pub base_amount: f32,
}

impl ResourceRequest {
    pub fn validate(&self) -> Result<(), ResourceRequestError> {
        if !self.base_amount.is_finite() || self.base_amount < 0.0 {
            return Err(ResourceRequestError::InvalidAmount);
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceRequestError {
    InvalidAmount,
}

impl Display for ResourceRequestError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InvalidAmount => {
                "La quantità di risorsa deve essere finita e non negativa"
            }
        };

        formatter.write_str(message)
    }
}

impl std::error::Error for ResourceRequestError {}

#[derive(Clone, Debug, PartialEq)]
pub struct ResourceResolution {
    pub request: ResourceRequest,
    pub effective_amount: f32,
}

impl ResourceResolution {
    pub fn new(
        request: ResourceRequest,
        effective_amount: f32,
    ) -> Result<Self, ResourceResolutionError> {
        let resolution = Self {
            request,
            effective_amount,
        };

        resolution.validate()?;

        Ok(resolution)
    }

    pub fn validate(&self) -> Result<(), ResourceResolutionError> {
        self.request
            .validate()
            .map_err(ResourceResolutionError::InvalidRequest)?;

        if !self.effective_amount.is_finite() || self.effective_amount < 0.0 {
            return Err(ResourceResolutionError::InvalidEffectiveAmount);
        }

        if self.effective_amount > self.request.base_amount {
            return Err(ResourceResolutionError::EffectiveExceedsRequestedAmount);
        }

        Ok(())
    }

    pub fn unapplied_amount(&self) -> f32 {
        self.request.base_amount - self.effective_amount
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceResolutionError {
    InvalidRequest(ResourceRequestError),
    InvalidEffectiveAmount,
    EffectiveExceedsRequestedAmount,
}

impl Display for ResourceResolutionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRequest(error) => {
                write!(formatter, "Richiesta di risorsa non valida: {error}")
            }
            Self::InvalidEffectiveAmount => {
                formatter.write_str(
                    "La quantità effettiva deve essere finita e non negativa",
                )
            }
            Self::EffectiveExceedsRequestedAmount => {
                formatter.write_str(
                    "La quantità effettiva non può superare quella richiesta",
                )
            }
        }
    }
}

impl std::error::Error for ResourceResolutionError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn mana_id() -> ContentId {
        ContentId::new("classic_1_12_1:resource.mana").unwrap()
    }

    fn energy_id() -> ContentId {
        ContentId::new("classic_1_12_1:resource.energy").unwrap()
    }

    fn valid_request(operation: ResourceOperation) -> ResourceRequest {
        ResourceRequest {
            source: None,
            target: Entity::from_bits(1),
            resource: mana_id(),
            operation,
            base_amount: 50.0,
        }
    }

    #[test]
    fn resource_pool_accepts_valid_values() {
        let pool = ResourcePool::new(50.0, 100.0).unwrap();

        assert_eq!(pool.current(), 50.0);
        assert_eq!(pool.maximum(), 100.0);
        assert_eq!(pool.remaining_capacity(), 50.0);
    }

    #[test]
    fn resource_pool_accepts_zero_values() {
        let pool = ResourcePool::new(0.0, 0.0).unwrap();

        assert_eq!(pool.current(), 0.0);
        assert_eq!(pool.maximum(), 0.0);
    }

    #[test]
    fn resource_pool_rejects_invalid_current_values() {
        for value in [-1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert_eq!(
                ResourcePool::new(value, 100.0),
                Err(ResourcePoolError::InvalidCurrent)
            );
        }
    }

    #[test]
    fn resource_pool_rejects_invalid_maximum_values() {
        for value in [-1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert_eq!(
                ResourcePool::new(50.0, value),
                Err(ResourcePoolError::InvalidMaximum)
            );
        }
    }

    #[test]
    fn resource_pool_rejects_current_above_maximum() {
        assert_eq!(
            ResourcePool::new(150.0, 100.0),
            Err(ResourcePoolError::CurrentExceedsMaximum)
        );
    }

    #[test]
    fn resource_pool_can_update_current_value() {
        let mut pool = ResourcePool::new(50.0, 100.0).unwrap();

        pool.set_current(75.0).unwrap();

        assert_eq!(pool.current(), 75.0);
        assert_eq!(pool.remaining_capacity(), 25.0);
    }

    #[test]
    fn resource_pool_rejects_invalid_current_updates() {
        let mut pool = ResourcePool::new(50.0, 100.0).unwrap();

        assert_eq!(
            pool.set_current(150.0),
            Err(ResourcePoolError::CurrentExceedsMaximum)
        );

        assert_eq!(pool.current(), 50.0);
    }

    #[test]
    fn resource_pool_can_update_maximum_value() {
        let mut pool = ResourcePool::new(50.0, 100.0).unwrap();

        pool.set_maximum(150.0).unwrap();

        assert_eq!(pool.maximum(), 150.0);
        assert_eq!(pool.remaining_capacity(), 100.0);
    }

    #[test]
    fn resource_pool_rejects_maximum_below_current() {
        let mut pool = ResourcePool::new(50.0, 100.0).unwrap();

        assert_eq!(
            pool.set_maximum(25.0),
            Err(ResourcePoolError::CurrentExceedsMaximum)
        );

        assert_eq!(pool.maximum(), 100.0);
    }

    #[test]
    fn combat_resources_store_independent_pools() {
        let mut resources = CombatResources::default();

        resources.insert(
            mana_id(),
            ResourcePool::new(80.0, 100.0).unwrap(),
        );

        resources.insert(
            energy_id(),
            ResourcePool::new(30.0, 100.0).unwrap(),
        );

        assert_eq!(resources.len(), 2);
        assert_eq!(resources.pool(&mana_id()).unwrap().current(), 80.0);
        assert_eq!(resources.pool(&energy_id()).unwrap().current(), 30.0);
    }

    #[test]
    fn combat_resources_can_remove_pools() {
        let mut resources = CombatResources::default();

        resources.insert(
            mana_id(),
            ResourcePool::new(80.0, 100.0).unwrap(),
        );

        assert!(resources.remove(&mana_id()).is_some());
        assert!(resources.pool(&mana_id()).is_none());
        assert!(resources.is_empty());
    }

    #[test]
    fn valid_resource_requests_pass_validation() {
        assert!(
            valid_request(ResourceOperation::Gain)
                .validate()
                .is_ok()
        );

        assert!(
            valid_request(ResourceOperation::Spend)
                .validate()
                .is_ok()
        );
    }

    #[test]
    fn resource_requests_reject_invalid_amounts() {
        for amount in [-1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let mut request = valid_request(ResourceOperation::Gain);
            request.base_amount = amount;

            assert_eq!(
                request.validate(),
                Err(ResourceRequestError::InvalidAmount)
            );
        }
    }

    #[test]
    fn resource_requests_accept_zero_amount() {
        let mut request = valid_request(ResourceOperation::Gain);
        request.base_amount = 0.0;

        assert!(request.validate().is_ok());
    }

    #[test]
    fn resource_resolution_supports_partial_changes() {
        let resolution = ResourceResolution::new(
            valid_request(ResourceOperation::Gain),
            30.0,
        )
        .unwrap();

        assert_eq!(resolution.effective_amount, 30.0);
        assert_eq!(resolution.unapplied_amount(), 20.0);
        assert!(resolution.validate().is_ok());
    }

    #[test]
    fn resource_resolution_supports_full_changes() {
        let resolution = ResourceResolution::new(
            valid_request(ResourceOperation::Spend),
            50.0,
        )
        .unwrap();

        assert_eq!(resolution.unapplied_amount(), 0.0);
    }

    #[test]
    fn resource_resolution_supports_zero_effective_changes() {
        let resolution = ResourceResolution::new(
            valid_request(ResourceOperation::Spend),
            0.0,
        )
        .unwrap();

        assert_eq!(resolution.unapplied_amount(), 50.0);
    }

    #[test]
    fn resource_resolution_rejects_invalid_effective_amounts() {
        for amount in [-1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert_eq!(
                ResourceResolution::new(
                    valid_request(ResourceOperation::Gain),
                    amount,
                ),
                Err(ResourceResolutionError::InvalidEffectiveAmount)
            );
        }
    }

    #[test]
    fn resource_resolution_rejects_effective_amount_above_request() {
        assert_eq!(
            ResourceResolution::new(
                valid_request(ResourceOperation::Gain),
                75.0,
            ),
            Err(ResourceResolutionError::EffectiveExceedsRequestedAmount)
        );
    }

    #[test]
    fn resource_resolution_rejects_invalid_requests() {
        let mut request = valid_request(ResourceOperation::Spend);
        request.base_amount = -10.0;

        assert_eq!(
            ResourceResolution::new(request, 0.0),
            Err(ResourceResolutionError::InvalidRequest(
                ResourceRequestError::InvalidAmount
            ))
        );
    }
}
