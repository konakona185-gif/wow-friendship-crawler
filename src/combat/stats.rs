use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use bevy::prelude::*;

use crate::content::ContentId;

#[derive(Component)]
pub struct CombatStats {
    pub current_health: f32,
    pub max_health: f32,
    pub resource: f32,
}

#[derive(Component, Clone, Debug, Default)]
pub struct CombatAttributes {
    values: HashMap<ContentId, f32>,
}

impl CombatAttributes {
    pub fn value(&self, stat: &ContentId) -> f32 {
        self.values.get(stat).copied().unwrap_or(0.0)
    }

    pub fn set(
        &mut self,
        stat: ContentId,
        value: f32,
    ) -> Result<(), CombatAttributeError> {
        if !value.is_finite() {
            return Err(CombatAttributeError::InvalidValue);
        }

        self.values.insert(stat, value);

        Ok(())
    }

    pub fn remove(&mut self, stat: &ContentId) -> Option<f32> {
        self.values.remove(stat)
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CombatAttributeError {
    InvalidValue,
}

impl Display for CombatAttributeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InvalidValue => {
                "Il valore dell'attributo deve essere finito"
            }
        };

        formatter.write_str(message)
    }
}

impl std::error::Error for CombatAttributeError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn armor_id() -> ContentId {
        ContentId::new("classic_1_12_1:stat.armor").unwrap()
    }

    #[test]
    fn missing_attributes_return_zero() {
        let attributes = CombatAttributes::default();

        assert_eq!(attributes.value(&armor_id()), 0.0);
    }

    #[test]
    fn attributes_can_be_set_and_retrieved() {
        let mut attributes = CombatAttributes::default();

        attributes.set(armor_id(), 500.0).unwrap();

        assert_eq!(attributes.value(&armor_id()), 500.0);
        assert_eq!(attributes.len(), 1);
    }

    #[test]
    fn attributes_accept_negative_finite_values() {
        let mut attributes = CombatAttributes::default();

        attributes.set(armor_id(), -50.0).unwrap();

        assert_eq!(attributes.value(&armor_id()), -50.0);
    }

    #[test]
    fn attributes_reject_non_finite_values() {
        for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let mut attributes = CombatAttributes::default();

            assert_eq!(
                attributes.set(armor_id(), value),
                Err(CombatAttributeError::InvalidValue)
            );

            assert!(attributes.is_empty());
        }
    }

    #[test]
    fn attributes_can_be_removed() {
        let mut attributes = CombatAttributes::default();
        let id = armor_id();

        attributes.set(id.clone(), 500.0).unwrap();

        assert_eq!(attributes.remove(&id), Some(500.0));
        assert_eq!(attributes.value(&id), 0.0);
        assert!(attributes.is_empty());
    }
}
