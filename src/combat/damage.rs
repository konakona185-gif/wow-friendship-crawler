use bevy::prelude::Entity;

use crate::content::ContentId;

use super::{CombatAttributes, CombatLevel};

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArmorMitigationError {
    InvalidIncomingAmount,
    InvalidArmor,
}

impl std::fmt::Display for ArmorMitigationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::InvalidIncomingAmount => {
                "Il danno in ingresso deve essere finito e non negativo"
            }
            Self::InvalidArmor => {
                "L'armatura deve essere un valore finito"
            }
        };

        formatter.write_str(message)
    }
}

impl std::error::Error for ArmorMitigationError {}

pub fn calculate_armor_mitigation(
    incoming_damage: f32,
    armor: f32,
    attacker_level: &CombatLevel,
) -> Result<DamageMitigationResult, ArmorMitigationError> {
    if !incoming_damage.is_finite() || incoming_damage < 0.0 {
        return Err(ArmorMitigationError::InvalidIncomingAmount);
    }

    if !armor.is_finite() {
        return Err(ArmorMitigationError::InvalidArmor);
    }

    if incoming_damage == 0.0 {
        return Ok(DamageMitigationResult {
            incoming_amount: 0.0,
            mitigated_amount: 0.0,
        });
    }

    let effective_armor = armor.max(0.0) as f64;
    let level_constant = 85.0 * attacker_level.value() as f64 + 400.0;

    let mitigation_fraction =
        (effective_armor / (effective_armor + level_constant))
            .clamp(0.0, 0.75);

    let mitigated_amount = incoming_damage * mitigation_fraction as f32;

    DamageMitigationResult::new(
        incoming_damage,
        mitigated_amount,
    )
    .map_err(|_| ArmorMitigationError::InvalidIncomingAmount)
}

pub fn calculate_armor_mitigation_from_attributes(
    incoming_damage: f32,
    armor_attribute: &ContentId,
    target_attributes: &CombatAttributes,
    attacker_level: &CombatLevel,
) -> Result<DamageMitigationResult, ArmorMitigationError> {
    let armor = target_attributes.value(armor_attribute);

    calculate_armor_mitigation(
        incoming_damage,
        armor,
        attacker_level,
    )
}

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

    #[test]
    fn armor_mitigation_returns_full_damage_without_armor() {
        let result =
            calculate_armor_mitigation(100.0, 0.0, &combat_level(60)).unwrap();

        assert_eq!(result.mitigated_amount, 0.0);
        assert_eq!(result.remaining_amount(), 100.0);
    }

    #[test]
    fn armor_mitigation_calculates_fifty_percent_at_level_sixty() {
        let result =
            calculate_armor_mitigation(100.0, 5500.0, &combat_level(60)).unwrap();

        assert_eq!(result.mitigated_amount, 50.0);
        assert_eq!(result.remaining_amount(), 50.0);
    }

    #[test]
    fn armor_mitigation_respects_seventy_five_percent_cap() {
        let result =
            calculate_armor_mitigation(1000.0, 50000.0, &combat_level(60)).unwrap();

        assert_eq!(result.mitigated_amount, 750.0);
        assert_eq!(result.remaining_amount(), 250.0);
    }

    #[test]
    fn armor_mitigation_clamps_negative_armor_to_zero() {
        let result =
            calculate_armor_mitigation(100.0, -500.0, &combat_level(60)).unwrap();

        assert_eq!(result.mitigated_amount, 0.0);
        assert_eq!(result.remaining_amount(), 100.0);
    }

    #[test]
    fn armor_mitigation_preserves_zero_damage() {
        let result =
            calculate_armor_mitigation(0.0, 5000.0, &combat_level(60)).unwrap();

        assert_eq!(result.mitigated_amount, 0.0);
        assert_eq!(result.remaining_amount(), 0.0);
    }

    #[test]
    fn armor_mitigation_scales_with_attacker_level() {
        let level_sixty =
            calculate_armor_mitigation(100.0, 5500.0, &combat_level(60)).unwrap();

        let level_seventy =
            calculate_armor_mitigation(100.0, 5500.0, &combat_level(70)).unwrap();

        assert!(level_sixty.mitigated_amount > level_seventy.mitigated_amount);
    }

    #[test]
    fn armor_mitigation_handles_extreme_finite_armor() {
        let result =
            calculate_armor_mitigation(1000.0, f32::MAX, &combat_level(60)).unwrap();

        assert_eq!(result.mitigated_amount, 750.0);
        assert_eq!(result.remaining_amount(), 250.0);
    }

    #[test]
    fn armor_mitigation_rejects_invalid_inputs() {
        assert_eq!(
            calculate_armor_mitigation(f32::NAN, 1000.0, &combat_level(60)),
            Err(ArmorMitigationError::InvalidIncomingAmount)
        );

        assert_eq!(
            calculate_armor_mitigation(100.0, f32::INFINITY, &combat_level(60)),
            Err(ArmorMitigationError::InvalidArmor)
        );
    }

    #[test]
    fn armor_mitigation_reads_combat_attributes() {
        let attributes = combat_attributes_with_armor(5500.0);

        let result = calculate_armor_mitigation_from_attributes(
            100.0,
            &armor_id(),
            &attributes,
            &combat_level(60),
        )
        .unwrap();

        assert_eq!(result.mitigated_amount, 50.0);
        assert_eq!(result.remaining_amount(), 50.0);
    }

    #[test]
    fn armor_mitigation_uses_zero_for_missing_attributes() {
        let attributes = CombatAttributes::default();

        let result = calculate_armor_mitigation_from_attributes(
            100.0,
            &armor_id(),
            &attributes,
            &combat_level(60),
        )
        .unwrap();

        assert_eq!(result.mitigated_amount, 0.0);
        assert_eq!(result.remaining_amount(), 100.0);
    }

    #[test]
    fn armor_mitigation_supports_custom_attribute_ids() {
        let custom_armor_id =
            ContentId::new("custom:armor_rating").unwrap();

        let mut attributes = CombatAttributes::default();
        attributes.set(custom_armor_id.clone(), 5500.0).unwrap();

        let result = calculate_armor_mitigation_from_attributes(
            100.0,
            &custom_armor_id,
            &attributes,
            &combat_level(60),
        )
        .unwrap();

        assert_eq!(result.mitigated_amount, 50.0);
    }

    #[test]
    fn armor_mitigation_clamps_negative_attribute_values() {
        let attributes = combat_attributes_with_armor(-500.0);

        let result = calculate_armor_mitigation_from_attributes(
            100.0,
            &armor_id(),
            &attributes,
            &combat_level(60),
        )
        .unwrap();

        assert_eq!(result.mitigated_amount, 0.0);
        assert_eq!(result.remaining_amount(), 100.0);
    }

    #[test]
    fn damage_resolution_combines_request_and_mitigation() {
        let resolution = DamageResolution::new(
            valid_request(),
            DamageMitigationResult::new(100.0, 25.0).unwrap(),
        )
        .unwrap();

        assert_eq!(resolution.final_amount(), 75.0);
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
