use bevy::prelude::Entity;

use crate::content::{
    ContentId,
    DamageSchoolDefinition,
    DamageSchoolMitigation,
    DamageSchoolValidationError,
};

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DamageMitigationError {
    InvalidIncomingAmount,
    InvalidMitigatedAmount,
    MitigationExceedsIncomingAmount,
    InvalidResistance,
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
            Self::InvalidResistance => {
                "La resistenza deve essere un valore finito"
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DamageSchoolMitigationError {
    InvalidRequest(DamageRequestError),
    SchoolMismatch,
    InvalidSchool(DamageSchoolValidationError),
    ArmorMitigation(ArmorMitigationError),
    ResistanceMitigation(DamageMitigationError),
    ResistanceMitigationNotImplemented,
}

impl std::fmt::Display for DamageSchoolMitigationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRequest(error) => {
                write!(formatter, "Richiesta di danno non valida: {error}")
            }
            Self::SchoolMismatch => {
                formatter.write_str(
                    "La scuola di danno non corrisponde alla richiesta originale",
                )
            }
            Self::InvalidSchool(error) => {
                write!(formatter, "Scuola di danno non valida: {error}")
            }
            Self::ArmorMitigation(error) => {
                write!(formatter, "Mitigazione dell'armatura non valida: {error}")
            }
            Self::ResistanceMitigation(error) => {
                write!(formatter, "Mitigazione della resistenza non valida: {error}")
            }
            Self::ResistanceMitigationNotImplemented => {
                formatter.write_str(
                    "La mitigazione tramite resistenza non è ancora implementata",
                )
            }
        }
    }
}

impl std::error::Error for DamageSchoolMitigationError {}

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

pub fn calculate_resistance_mitigation(
    incoming_damage: f32,
    resistance: f32,
    attacker_level: &CombatLevel,
) -> Result<DamageMitigationResult, DamageMitigationError> {
    if !incoming_damage.is_finite() || incoming_damage < 0.0 {
        return Err(DamageMitigationError::InvalidIncomingAmount);
    }

    if !resistance.is_finite() {
        return Err(DamageMitigationError::InvalidResistance);
    }

    if incoming_damage == 0.0 {
        return Ok(DamageMitigationResult {
            incoming_amount: 0.0,
            mitigated_amount: 0.0,
        });
    }

    let effective_level = attacker_level.value().max(20) as f64;
    let effective_resistance = resistance.max(0.0) as f64;

    let mitigation_fraction = (
        effective_resistance / (effective_level * 5.0) * 0.75
    )
        .clamp(0.0, 0.75);

    let mitigated_amount = incoming_damage * mitigation_fraction as f32;

    DamageMitigationResult::new(
        incoming_damage,
        mitigated_amount,
    )
}

fn calculate_component_mitigation(
    incoming_damage: f32,
    component: &DamageSchoolMitigation,
    armor_attribute: &ContentId,
    target_attributes: &CombatAttributes,
    attacker_level: &CombatLevel,
) -> Result<DamageMitigationResult, DamageSchoolMitigationError> {
    match component {
        DamageSchoolMitigation::Unmitigated => {
            Ok(DamageMitigationResult {
                incoming_amount: incoming_damage,
                mitigated_amount: 0.0,
            })
        }
        DamageSchoolMitigation::Armor => {
            calculate_armor_mitigation_from_attributes(
                incoming_damage,
                armor_attribute,
                target_attributes,
                attacker_level,
            )
            .map_err(DamageSchoolMitigationError::ArmorMitigation)
        }
        DamageSchoolMitigation::Resistance { stat } => {
            calculate_resistance_mitigation(
                incoming_damage,
                target_attributes.value(stat),
                attacker_level,
            )
            .map_err(DamageSchoolMitigationError::ResistanceMitigation)
        }
        DamageSchoolMitigation::Composite { .. } => {
            Err(DamageSchoolMitigationError::InvalidSchool(
                DamageSchoolValidationError::NestedComposite,
            ))
        }
    }
}

pub fn calculate_damage_mitigation(
    request: &DamageRequest,
    school: &DamageSchoolDefinition,
    armor_attribute: &ContentId,
    target_attributes: &CombatAttributes,
    attacker_level: &CombatLevel,
) -> Result<DamageMitigationResult, DamageSchoolMitigationError> {
    request
        .validate()
        .map_err(DamageSchoolMitigationError::InvalidRequest)?;

    school
        .validate()
        .map_err(DamageSchoolMitigationError::InvalidSchool)?;

    if request.school != school.metadata.id {
        return Err(DamageSchoolMitigationError::SchoolMismatch);
    }

    match &school.mitigation {
        DamageSchoolMitigation::Composite { components } => {
            let mut best_result = DamageMitigationResult {
                incoming_amount: request.base_amount,
                mitigated_amount: 0.0,
            };

            for component in components {
                let result = calculate_component_mitigation(
                    request.base_amount,
                    component,
                    armor_attribute,
                    target_attributes,
                    attacker_level,
                )?;

                if result.mitigated_amount > best_result.mitigated_amount {
                    best_result = result;
                }
            }

            Ok(best_result)
        }
        component => calculate_component_mitigation(
            request.base_amount,
            component,
            armor_attribute,
            target_attributes,
            attacker_level,
        ),
    }
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
    use crate::content::ContentMetadata;

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

    fn resistance_id(name: &str) -> ContentId {
        ContentId::new(format!("classic_1_12_1:stat.{name}")).unwrap()
    }

    fn combat_attributes_with_armor(armor: f32) -> CombatAttributes {
        let mut attributes = CombatAttributes::default();
        attributes.set(armor_id(), armor).unwrap();
        attributes
    }

    fn combat_attributes_with_resistance(
        stat: &ContentId,
        resistance: f32,
    ) -> CombatAttributes {
        let mut attributes = CombatAttributes::default();
        attributes.set(stat.clone(), resistance).unwrap();
        attributes
    }

    fn resistance_component(name: &str) -> DamageSchoolMitigation {
        DamageSchoolMitigation::Resistance {
            stat: resistance_id(name),
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
    fn resistance_mitigation_calculates_twenty_five_percent_at_level_sixty() {
        let result =
            calculate_resistance_mitigation(100.0, 100.0, &combat_level(60))
                .unwrap();

        assert_eq!(result.mitigated_amount, 25.0);
        assert_eq!(result.remaining_amount(), 75.0);
    }

    #[test]
    fn resistance_mitigation_calculates_fifty_percent_at_level_sixty() {
        let result =
            calculate_resistance_mitigation(100.0, 200.0, &combat_level(60))
                .unwrap();

        assert_eq!(result.mitigated_amount, 50.0);
        assert_eq!(result.remaining_amount(), 50.0);
    }

    #[test]
    fn resistance_mitigation_respects_seventy_five_percent_cap() {
        let result =
            calculate_resistance_mitigation(1000.0, 500.0, &combat_level(60))
                .unwrap();

        assert_eq!(result.mitigated_amount, 750.0);
        assert_eq!(result.remaining_amount(), 250.0);
    }

    #[test]
    fn resistance_mitigation_scales_with_attacker_level() {
        let level_sixty =
            calculate_resistance_mitigation(100.0, 150.0, &combat_level(60))
                .unwrap();

        let level_seventy =
            calculate_resistance_mitigation(100.0, 150.0, &combat_level(70))
                .unwrap();

        assert!(level_sixty.mitigated_amount > level_seventy.mitigated_amount);
    }

    #[test]
    fn resistance_mitigation_uses_minimum_caster_level_of_twenty() {
        let level_ten =
            calculate_resistance_mitigation(100.0, 100.0, &combat_level(10))
                .unwrap();

        let level_twenty =
            calculate_resistance_mitigation(100.0, 100.0, &combat_level(20))
                .unwrap();

        assert_eq!(level_ten, level_twenty);
    }

    #[test]
    fn resistance_mitigation_clamps_negative_resistance_to_zero() {
        let result =
            calculate_resistance_mitigation(100.0, -50.0, &combat_level(60))
                .unwrap();

        assert_eq!(result.mitigated_amount, 0.0);
        assert_eq!(result.remaining_amount(), 100.0);
    }

    #[test]
    fn resistance_mitigation_preserves_zero_damage() {
        let result =
            calculate_resistance_mitigation(0.0, 300.0, &combat_level(60))
                .unwrap();

        assert_eq!(result.mitigated_amount, 0.0);
        assert_eq!(result.remaining_amount(), 0.0);
    }

    #[test]
    fn resistance_mitigation_rejects_invalid_inputs() {
        assert_eq!(
            calculate_resistance_mitigation(
                f32::NAN,
                100.0,
                &combat_level(60)
            ),
            Err(DamageMitigationError::InvalidIncomingAmount)
        );

        assert_eq!(
            calculate_resistance_mitigation(
                100.0,
                f32::INFINITY,
                &combat_level(60)
            ),
            Err(DamageMitigationError::InvalidResistance)
        );
    }

    #[test]
    fn resistance_mitigation_reads_combat_attributes() {
        let stat = resistance_id("fire_resistance");
        let attributes = combat_attributes_with_resistance(&stat, 200.0);
        let school = valid_school(
            DamageSchoolMitigation::Resistance {
                stat: stat.clone(),
            },
        );

        let result = calculate_damage_mitigation(
            &valid_request(),
            &school,
            &armor_id(),
            &attributes,
            &combat_level(60),
        )
        .unwrap();

        assert_eq!(result.mitigated_amount, 50.0);
        assert_eq!(result.remaining_amount(), 50.0);
    }

    #[test]
    fn resistance_mitigation_uses_zero_for_missing_attributes() {
        let school = valid_school(resistance_component("fire_resistance"));

        let result = calculate_damage_mitigation(
            &valid_request(),
            &school,
            &armor_id(),
            &CombatAttributes::default(),
            &combat_level(60),
        )
        .unwrap();

        assert_eq!(result.mitigated_amount, 0.0);
        assert_eq!(result.remaining_amount(), 100.0);
    }

    #[test]
    fn unmitigated_school_preserves_full_damage() {
        let request = valid_request();
        let school = valid_school(DamageSchoolMitigation::Unmitigated);

        let result = calculate_damage_mitigation(
            &request,
            &school,
            &armor_id(),
            &combat_attributes_with_armor(5500.0),
            &combat_level(60),
        )
        .unwrap();

        assert_eq!(result.mitigated_amount, 0.0);
        assert_eq!(result.remaining_amount(), 100.0);
    }

    #[test]
    fn armor_school_uses_armor_mitigation() {
        let request = valid_request();
        let school = valid_school(DamageSchoolMitigation::Armor);

        let result = calculate_damage_mitigation(
            &request,
            &school,
            &armor_id(),
            &combat_attributes_with_armor(5500.0),
            &combat_level(60),
        )
        .unwrap();

        assert_eq!(result.mitigated_amount, 50.0);
        assert_eq!(result.remaining_amount(), 50.0);
    }

    #[test]
    fn composite_school_uses_the_highest_resistance_without_stacking() {
        let request = valid_request();

        let school = valid_school(
            DamageSchoolMitigation::Composite {
                components: vec![
                    resistance_component("frost_resistance"),
                    resistance_component("shadow_resistance"),
                ],
            },
        );

        let mut attributes = CombatAttributes::default();
        attributes
            .set(resistance_id("frost_resistance"), 100.0)
            .unwrap();
        attributes
            .set(resistance_id("shadow_resistance"), 200.0)
            .unwrap();

        let result = calculate_damage_mitigation(
            &request,
            &school,
            &armor_id(),
            &attributes,
            &combat_level(60),
        )
        .unwrap();

        assert_eq!(result.mitigated_amount, 50.0);
        assert_eq!(result.remaining_amount(), 50.0);
    }

    #[test]
    fn composite_school_compares_armor_and_magic_resistance_by_mitigation() {
        let request = valid_request();

        let school = valid_school(
            DamageSchoolMitigation::Composite {
                components: vec![
                    DamageSchoolMitigation::Armor,
                    resistance_component("holy_resistance"),
                ],
            },
        );

        let mut attributes = combat_attributes_with_armor(5500.0);
        attributes
            .set(resistance_id("holy_resistance"), 150.0)
            .unwrap();

        let result = calculate_damage_mitigation(
            &request,
            &school,
            &armor_id(),
            &attributes,
            &combat_level(60),
        )
        .unwrap();

        assert_eq!(result.mitigated_amount, 50.0);
        assert_eq!(result.remaining_amount(), 50.0);
    }

    #[test]
    fn composite_school_does_not_add_armor_and_resistance_mitigation() {
        let request = valid_request();

        let school = valid_school(
            DamageSchoolMitigation::Composite {
                components: vec![
                    DamageSchoolMitigation::Armor,
                    resistance_component("fire_resistance"),
                ],
            },
        );

        let mut attributes = combat_attributes_with_armor(5500.0);
        attributes
            .set(resistance_id("fire_resistance"), 300.0)
            .unwrap();

        let result = calculate_damage_mitigation(
            &request,
            &school,
            &armor_id(),
            &attributes,
            &combat_level(60),
        )
        .unwrap();

        assert_eq!(result.mitigated_amount, 75.0);
        assert_eq!(result.remaining_amount(), 25.0);
    }

    #[test]
    fn damage_mitigation_rejects_mismatched_school_ids() {
        let request = valid_request();
        let mut school = valid_school(DamageSchoolMitigation::Unmitigated);
        school.metadata.id =
            ContentId::new("classic_1_12_1:school.fire").unwrap();

        assert_eq!(
            calculate_damage_mitigation(
                &request,
                &school,
                &armor_id(),
                &CombatAttributes::default(),
                &combat_level(60),
            ),
            Err(DamageSchoolMitigationError::SchoolMismatch)
        );
    }

    #[test]
    fn damage_mitigation_rejects_invalid_requests() {
        let mut request = valid_request();
        request.base_amount = -10.0;
        let school = valid_school(DamageSchoolMitigation::Unmitigated);

        assert_eq!(
            calculate_damage_mitigation(
                &request,
                &school,
                &armor_id(),
                &CombatAttributes::default(),
                &combat_level(60),
            ),
            Err(DamageSchoolMitigationError::InvalidRequest(
                DamageRequestError::InvalidAmount
            ))
        );
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
        let stat = resistance_id("fire_resistance");
        let attributes = combat_attributes_with_resistance(&stat, 200.0);

        let resolution = DamageResolution::from_school_mitigation(
            valid_request(),
            &valid_school(
                DamageSchoolMitigation::Resistance { stat },
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
