use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

use super::{ContentMetadata, ContentMetadataError};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CreatureDefinition {
    pub metadata: ContentMetadata,
    pub health: f32,
    pub attack_damage: f32,
    pub attack_interval_seconds: f32,
    pub melee_range: f32,
    pub aggro_range: f32,
    pub leash_range: f32,
    pub movement_speed: f32,
}

impl CreatureDefinition {
    pub fn validate(&self) -> Result<(), CreatureValidationError> {
        self.metadata
            .validate()
            .map_err(CreatureValidationError::InvalidMetadata)?;

        if !self.health.is_finite() || self.health <= 0.0 {
            return Err(CreatureValidationError::InvalidHealth);
        }

        if !self.attack_damage.is_finite() || self.attack_damage <= 0.0 {
            return Err(CreatureValidationError::InvalidAttackDamage);
        }

        if !self.attack_interval_seconds.is_finite() || self.attack_interval_seconds <= 0.0 {
            return Err(CreatureValidationError::InvalidAttackInterval);
        }

        if !self.melee_range.is_finite() || self.melee_range <= 0.0 {
            return Err(CreatureValidationError::InvalidMeleeRange);
        }

        if !self.aggro_range.is_finite() || self.aggro_range <= 0.0 {
            return Err(CreatureValidationError::InvalidAggroRange);
        }

        if !self.leash_range.is_finite() || self.leash_range < self.aggro_range {
            return Err(CreatureValidationError::InvalidLeashRange);
        }

        if !self.movement_speed.is_finite() || self.movement_speed <= 0.0 {
            return Err(CreatureValidationError::InvalidMovementSpeed);
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CreatureValidationError {
    InvalidMetadata(ContentMetadataError),
    InvalidHealth,
    InvalidAttackDamage,
    InvalidAttackInterval,
    InvalidMeleeRange,
    InvalidAggroRange,
    InvalidLeashRange,
    InvalidMovementSpeed,
}

impl Display for CreatureValidationError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InvalidMetadata(error) => {
                return write!(formatter, "Metadati della creatura non validi: {error}");
            }
            Self::InvalidHealth => "La salute della creatura deve essere finita e positiva",
            Self::InvalidAttackDamage => "Il danno della creatura deve essere finito e positivo",
            Self::InvalidAttackInterval => {
                "L'intervallo degli attacchi deve essere finito e positivo"
            }
            Self::InvalidMeleeRange => "La gittata degli attacchi deve essere finita e positiva",
            Self::InvalidAggroRange => "Il raggio di aggro deve essere finito e positivo",
            Self::InvalidLeashRange => {
                "Il raggio di leash deve essere finito e almeno pari al raggio di aggro"
            }
            Self::InvalidMovementSpeed => "La velocità di movimento deve essere finita e positiva",
        };

        formatter.write_str(message)
    }
}

impl std::error::Error for CreatureValidationError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::ContentId;

    fn valid_creature() -> CreatureDefinition {
        CreatureDefinition {
            metadata: ContentMetadata {
                id: ContentId::new("classic_1_12_1:creature.wolf_companion").unwrap(),
                pack_id: ContentId::new("pack:classic_1_12_1").unwrap(),
                name: "Wolf Companion".to_string(),
                description: String::new(),
                icon: None,
                tags: vec!["summon".to_string()],
            },
            health: 150.0,
            attack_damage: 12.0,
            attack_interval_seconds: 2.0,
            melee_range: 2.5,
            aggro_range: 15.0,
            leash_range: 40.0,
            movement_speed: 5.0,
        }
    }

    #[test]
    fn valid_creature_passes_validation() {
        assert!(valid_creature().validate().is_ok());
    }

    #[test]
    fn creature_rejects_invalid_health_and_damage() {
        let mut creature = valid_creature();
        creature.health = 0.0;

        assert_eq!(
            creature.validate(),
            Err(CreatureValidationError::InvalidHealth)
        );

        let mut creature = valid_creature();
        creature.attack_damage = f32::INFINITY;

        assert_eq!(
            creature.validate(),
            Err(CreatureValidationError::InvalidAttackDamage)
        );
    }

    #[test]
    fn creature_rejects_invalid_attack_timing_and_range() {
        let mut creature = valid_creature();
        creature.attack_interval_seconds = 0.0;

        assert_eq!(
            creature.validate(),
            Err(CreatureValidationError::InvalidAttackInterval)
        );

        let mut creature = valid_creature();
        creature.melee_range = f32::NAN;

        assert_eq!(
            creature.validate(),
            Err(CreatureValidationError::InvalidMeleeRange)
        );
    }

    #[test]
    fn creature_rejects_invalid_aggro_and_leash_ranges() {
        let mut creature = valid_creature();
        creature.aggro_range = 0.0;

        assert_eq!(
            creature.validate(),
            Err(CreatureValidationError::InvalidAggroRange)
        );

        let mut creature = valid_creature();
        creature.leash_range = 10.0;

        assert_eq!(
            creature.validate(),
            Err(CreatureValidationError::InvalidLeashRange)
        );
    }

    #[test]
    fn creature_rejects_invalid_movement_speed() {
        let mut creature = valid_creature();
        creature.movement_speed = -1.0;

        assert_eq!(
            creature.validate(),
            Err(CreatureValidationError::InvalidMovementSpeed)
        );
    }

    #[test]
    fn creature_supports_ron_serialization() {
        let original = valid_creature();

        let serialized = ron::to_string(&original).unwrap();
        let restored: CreatureDefinition = ron::from_str(&serialized).unwrap();

        assert_eq!(original, restored);
        assert!(restored.validate().is_ok());
    }
}
