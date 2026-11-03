use bevy::prelude::Entity;

use crate::content::{
    ContentId,
    ContentMetadata,
    DamageSchoolDefinition,
    DamageSchoolMitigation,
};

use super::{CombatAttributes, CombatLevel};

mod critical;
mod mitigation;

pub use critical::{
    calculate_critical_strike,
    CriticalStrikeError,
    CriticalStrikeResult,
};

pub use mitigation::{
    calculate_armor_mitigation,
    calculate_armor_mitigation_from_attributes,
    calculate_damage_mitigation,
    ArmorMitigationError,
    DamageMitigationError,
    DamageMitigationResult,
    DamageSchoolMitigationError,
};

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

#[derive(Clone, Debug, PartialEq)]
pub struct DamageResolution {
    pub request: DamageRequest,
    pub mitigation: DamageMitigationResult,
}

impl DamageResolution {
    pub fn new(
        request: DamageRequest,
        mitigation: DamageMitigationResult,
    ) -> Result<Self, DamageResolutionError> {
        let resolution = Self {
            request,
            mitigation,
        };

        resolution.validate()?;

        Ok(resolution)
    }

    pub fn from_school_mitigation(
        request: DamageRequest,
        school: &DamageSchoolDefinition,
        armor_attribute: &ContentId,
        target_attributes: &CombatAttributes,
        attacker_level: &CombatLevel,
    ) -> Result<Self, DamageResolutionError> {
        let mitigation = calculate_damage_mitigation(
            &request,
            school,
            armor_attribute,
            target_attributes,
            attacker_level,
        )
        .map_err(DamageResolutionError::MitigationCalculation)?;

        Self::new(request, mitigation)
    }

    pub fn validate(&self) -> Result<(), DamageResolutionError> {
        self.request
            .validate()
            .map_err(DamageResolutionError::InvalidRequest)?;

        self.mitigation
            .validate()
            .map_err(DamageResolutionError::InvalidMitigation)?;

        if self.request.base_amount != self.mitigation.incoming_amount {
            return Err(DamageResolutionError::IncomingAmountMismatch);
        }

        Ok(())
    }

    pub fn final_amount(&self) -> f32 {
        self.mitigation.remaining_amount()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DamageResolutionError {
    InvalidRequest(DamageRequestError),
    InvalidMitigation(DamageMitigationError),
    MitigationCalculation(DamageSchoolMitigationError),
    IncomingAmountMismatch,
}

impl std::fmt::Display for DamageResolutionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRequest(error) => {
                write!(formatter, "Richiesta di danno non valida: {error}")
            }
            Self::InvalidMitigation(error) => {
                write!(formatter, "Mitigazione non valida: {error}")
            }
            Self::MitigationCalculation(error) => {
                write!(formatter, "Calcolo della mitigazione non riuscito: {error}")
            }
            Self::IncomingAmountMismatch => {
                formatter.write_str(
                    "Il danno in ingresso non corrisponde alla richiesta originale",
                )
            }
        }
    }
}

impl std::error::Error for DamageResolutionError {}

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

    fn valid_school(mitigation: DamageSchoolMitigation) -> DamageSchoolDefinition {
        DamageSchoolDefinition {
            metadata: ContentMetadata {
                id: ContentId::new("classic_1_12_1:school.physical").unwrap(),
                pack_id: ContentId::new("pack:classic_1_12_1").unwrap(),
                name: "Physical".to_string(),
                description: String::new(),
                icon: None,
                tags: Vec::new(),
            },
            mitigation,
        }
    }

    fn combat_level(value: u32) -> CombatLevel {
        CombatLevel::new(value).unwrap()
    }

    fn armor_id() -> ContentId {
        ContentId::new("classic_1_12_1:stat.armor").unwrap()
    }

    fn combat_attributes_with_armor(armor: f32) -> CombatAttributes {
        let mut attributes = CombatAttributes::default();
        attributes.set(armor_id(), armor).unwrap();
        attributes
    }

    fn valid_damage_resolution() -> DamageResolution {
        DamageResolution::new(
            valid_request(),
            DamageMitigationResult::new(100.0, 25.0).unwrap(),
        )
        .unwrap()
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
    fn damage_resolution_combines_request_and_mitigation() {
        let resolution = valid_damage_resolution();

        assert_eq!(resolution.final_amount(), 75.0);
        assert!(resolution.validate().is_ok());
    }

    #[test]
    fn damage_resolution_builds_from_school_mitigation() {
        let resolution = DamageResolution::from_school_mitigation(
            valid_request(),
            &valid_school(DamageSchoolMitigation::Armor),
            &armor_id(),
            &combat_attributes_with_armor(5500.0),
            &combat_level(60),
        )
        .unwrap();

        assert_eq!(resolution.mitigation.mitigated_amount, 50.0);
        assert_eq!(resolution.final_amount(), 50.0);
        assert!(resolution.validate().is_ok());
    }

    #[test]
    fn damage_resolution_preserves_unmitigated_school_damage() {
        let resolution = DamageResolution::from_school_mitigation(
            valid_request(),
            &valid_school(DamageSchoolMitigation::Unmitigated),
            &armor_id(),
            &combat_attributes_with_armor(5500.0),
            &combat_level(60),
        )
        .unwrap();

        assert_eq!(resolution.mitigation.mitigated_amount, 0.0);
        assert_eq!(resolution.final_amount(), 100.0);
    }

    #[test]
    fn damage_resolution_applies_resistance_mitigation() {
        let resistance_id =
            ContentId::new("classic_1_12_1:stat.fire_resistance").unwrap();

        let mut attributes = CombatAttributes::default();
        attributes.set(resistance_id.clone(), 200.0).unwrap();

        let resolution = DamageResolution::from_school_mitigation(
            valid_request(),
            &valid_school(
                DamageSchoolMitigation::Resistance {
                    stat: resistance_id,
                },
            ),
            &armor_id(),
            &attributes,
            &combat_level(60),
        )
        .unwrap();

        assert_eq!(resolution.mitigation.mitigated_amount, 50.0);
        assert_eq!(resolution.final_amount(), 50.0);
        assert!(resolution.validate().is_ok());
    }

    #[test]
    fn damage_resolution_rejects_mismatched_incoming_amounts() {
        assert_eq!(
            DamageResolution::new(
                valid_request(),
                DamageMitigationResult::new(80.0, 20.0).unwrap(),
            ),
            Err(DamageResolutionError::IncomingAmountMismatch)
        );
    }

    #[test]
    fn damage_resolution_rejects_invalid_requests() {
        let mut request = valid_request();
        request.base_amount = -10.0;

        assert_eq!(
            DamageResolution::new(
                request,
                DamageMitigationResult::new(100.0, 0.0).unwrap(),
            ),
            Err(DamageResolutionError::InvalidRequest(
                DamageRequestError::InvalidAmount
            ))
        );
    }
}
