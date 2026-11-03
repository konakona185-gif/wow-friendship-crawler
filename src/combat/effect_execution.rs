use std::fmt::{Display, Formatter};

use crate::content::{
    ContentDatabase,
    ContentId,
    ContentIdError,
};

use super::{
    CombatAttributes,
    CombatLevel,
    CombatResources,
    DamageResolution,
    HealingRequestError,
    HealingResolution,
    HealingResolutionError,
    HealthPool,
    HealthPoolError,
    ResourceApplicationError,
    ResourceOperation,
    ResourceRequest,
    ResourceResolution,
    ResolvedEffect,
    ResolvedEffectOperation,
};

pub struct EffectExecutionContext<'a> {
    pub database: &'a ContentDatabase,
    pub attacker_level: Option<&'a CombatLevel>,
    pub target_attributes: Option<&'a CombatAttributes>,
    pub target_health: Option<&'a mut HealthPool>,
    pub target_resources: Option<&'a mut CombatResources>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum EffectExecutionResult {
    Damage {
        resolution: DamageResolution,
        applied_amount: f32,
    },
    Healing {
        resolution: HealingResolution,
        applied_amount: f32,
    },
    Resource(ResourceResolution),
}

pub fn execute_resolved_effect(
    effect: &ResolvedEffect,
    mut context: EffectExecutionContext<'_>,
) -> Result<EffectExecutionResult, EffectExecutionError> {
    match &effect.operation {
        ResolvedEffectOperation::Damage(request) => {
            if request.source != effect.source
                || request.target != effect.target
            {
                return Err(EffectExecutionError::ContextMismatch);
            }

            let school = context
                .database
                .damage_school(&request.school)
                .ok_or_else(|| {
                    EffectExecutionError::MissingDamageSchool(
                        request.school.clone(),
                    )
                })?;

            let armor_attribute = ContentId::new(
                "classic_1_12_1:stat.armor",
            )
            .map_err(EffectExecutionError::InvalidArmorAttributeId)?;

            if context
                .database
                .combat_attribute(&armor_attribute)
                .is_none()
            {
                return Err(
                    EffectExecutionError::MissingArmorAttribute(
                        armor_attribute,
                    ),
                );
            }

            let attacker_level = context
                .attacker_level
                .ok_or(EffectExecutionError::MissingAttackerLevel)?;

            let target_attributes = context
                .target_attributes
                .ok_or(EffectExecutionError::MissingTargetAttributes)?;

            let target_health = context
                .target_health
                .as_deref_mut()
                .ok_or(EffectExecutionError::MissingTargetHealth)?;

            let resolution = DamageResolution::from_school_mitigation(
                request.clone(),
                school,
                &armor_attribute,
                target_attributes,
                attacker_level,
            )
            .map_err(EffectExecutionError::InvalidDamageResolution)?;

            let applied_amount = target_health
                .apply_damage_resolution(&resolution)
                .map_err(EffectExecutionError::HealthApplication)?;

            Ok(EffectExecutionResult::Damage {
                resolution,
                applied_amount,
            })
        }
        ResolvedEffectOperation::Heal(request) => {
            if request.source != effect.source
                || request.target != effect.target
            {
                return Err(EffectExecutionError::ContextMismatch);
            }

            request
                .validate()
                .map_err(EffectExecutionError::InvalidHealingRequest)?;

            let target_health = context
                .target_health
                .as_deref_mut()
                .ok_or(EffectExecutionError::MissingTargetHealth)?;

            let effective_amount = target_health
                .remaining_capacity()
                .min(request.base_amount);

            let resolution = HealingResolution::new(
                request.clone(),
                effective_amount,
            )
            .map_err(EffectExecutionError::InvalidHealingResolution)?;

            let applied_amount = target_health
                .apply_healing_resolution(&resolution)
                .map_err(EffectExecutionError::HealthApplication)?;

            Ok(EffectExecutionResult::Healing {
                resolution,
                applied_amount,
            })
        }
        ResolvedEffectOperation::GainResource(request) => {
            execute_resource_request(
                effect,
                request,
                ResourceOperation::Gain,
                &mut context,
            )
        }
        ResolvedEffectOperation::SpendResource(request) => {
            execute_resource_request(
                effect,
                request,
                ResourceOperation::Spend,
                &mut context,
            )
        }
        _ => Err(EffectExecutionError::UnsupportedOperation),
    }
}

fn execute_resource_request(
    effect: &ResolvedEffect,
    request: &ResourceRequest,
    expected_operation: ResourceOperation,
    context: &mut EffectExecutionContext<'_>,
) -> Result<EffectExecutionResult, EffectExecutionError> {
    if request.source != effect.source
        || request.target != effect.target
    {
        return Err(EffectExecutionError::ContextMismatch);
    }

    if request.operation != expected_operation {
        return Err(EffectExecutionError::ResourceOperationMismatch);
    }

    if context.database.resource(&request.resource).is_none() {
        return Err(
            EffectExecutionError::MissingResourceDefinition(
                request.resource.clone(),
            ),
        );
    }

    let target_resources = context
        .target_resources
        .as_deref_mut()
        .ok_or(EffectExecutionError::MissingTargetResources)?;

    let resolution = target_resources
        .apply_request(request.clone())
        .map_err(EffectExecutionError::ResourceApplication)?;

    Ok(EffectExecutionResult::Resource(resolution))
}

#[derive(Clone, Debug, PartialEq)]
pub enum EffectExecutionError {
    ContextMismatch,
    MissingDamageSchool(ContentId),
    MissingArmorAttribute(ContentId),
    MissingResourceDefinition(ContentId),
    MissingAttackerLevel,
    MissingTargetAttributes,
    MissingTargetHealth,
    MissingTargetResources,
    InvalidArmorAttributeId(ContentIdError),
    InvalidDamageResolution(super::DamageResolutionError),
    InvalidHealingRequest(HealingRequestError),
    InvalidHealingResolution(HealingResolutionError),
    HealthApplication(HealthPoolError),
    ResourceOperationMismatch,
    ResourceApplication(ResourceApplicationError),
    UnsupportedOperation,
}

impl Display for EffectExecutionError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ContextMismatch => {
                formatter.write_str(
                    "La sorgente o il bersaglio dell'operazione non corrispondono all'effetto",
                )
            }
            Self::MissingDamageSchool(id) => {
                write!(
                    formatter,
                    "Scuola di danno non registrata: {}",
                    id.as_str()
                )
            }
            Self::MissingArmorAttribute(id) => {
                write!(
                    formatter,
                    "Attributo dell'armatura non registrato: {}",
                    id.as_str()
                )
            }
            Self::MissingResourceDefinition(id) => {
                write!(
                    formatter,
                    "Definizione della risorsa non registrata: {}",
                    id.as_str()
                )
            }
            Self::MissingAttackerLevel => {
                formatter.write_str(
                    "Il calcolo del danno richiede il livello dell'attaccante",
                )
            }
            Self::MissingTargetAttributes => {
                formatter.write_str(
                    "Il calcolo del danno richiede gli attributi del bersaglio",
                )
            }
            Self::MissingTargetHealth => {
                formatter.write_str(
                    "Il bersaglio non possiede una HealthPool",
                )
            }
            Self::MissingTargetResources => {
                formatter.write_str(
                    "Il bersaglio non possiede CombatResources",
                )
            }
            Self::InvalidArmorAttributeId(error) => {
                write!(
                    formatter,
                    "Identificativo dell'armatura non valido: {error}"
                )
            }
            Self::InvalidDamageResolution(error) => {
                write!(
                    formatter,
                    "Risoluzione del danno non valida: {error}"
                )
            }
            Self::InvalidHealingRequest(error) => {
                write!(
                    formatter,
                    "Richiesta di guarigione non valida: {error}"
                )
            }
            Self::InvalidHealingResolution(error) => {
                write!(
                    formatter,
                    "Risoluzione della guarigione non valida: {error}"
                )
            }
            Self::HealthApplication(error) => {
                write!(
                    formatter,
                    "Applicazione alla HealthPool non riuscita: {error}"
                )
            }
            Self::ResourceOperationMismatch => {
                formatter.write_str(
                    "Il tipo di operazione della risorsa non corrisponde all'effetto",
                )
            }
            Self::ResourceApplication(error) => {
                write!(
                    formatter,
                    "Applicazione della risorsa non riuscita: {error}"
                )
            }
            Self::UnsupportedOperation => {
                formatter.write_str(
                    "L'operazione dell'effetto non è ancora supportata",
                )
            }
        }
    }
}

impl std::error::Error for EffectExecutionError {}

#[cfg(test)]
mod tests {
    use super::*;

    use bevy::prelude::Entity;

    use crate::combat::{
        CombatAttributes,
        CombatLevel,
        CombatResources,
        EffectTrigger,
        ResourcePool,
    };

    use crate::content::{
        CombatAttributeDefinition,
        CombatAttributeValueKind,
        ContentMetadata,
        ContentPackManifest,
        DamageSchoolDefinition,
        DamageSchoolMitigation,
        ResourceDefinition,
    };

    fn content_id(value: &str) -> ContentId {
        ContentId::new(
            format!("classic_1_12_1:{value}"),
        )
        .unwrap()
    }

    fn metadata(id: &str, name: &str) -> ContentMetadata {
        ContentMetadata {
            id: content_id(id),
            pack_id: ContentId::new(
                "pack:classic_1_12_1",
            )
            .unwrap(),
            name: name.to_string(),
            description: String::new(),
            icon: None,
            tags: Vec::new(),
        }
    }

    fn database_with_school(
        mitigation: DamageSchoolMitigation,
    ) -> ContentDatabase {
        let mut database = ContentDatabase::new();

        database.register_pack(ContentPackManifest {
            id: ContentId::new(
                "pack:classic_1_12_1",
            )
            .unwrap(),
            name: "Classic 1.12.1".to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            dependencies: Vec::new(),
        })
        .unwrap();

        for (id, name) in [
            ("stat.armor", "Armor"),
            ("stat.fire_resistance", "Fire Resistance"),
        ] {
            database
                .register_combat_attribute(CombatAttributeDefinition {
                    metadata: metadata(id, name),
                    value_kind: CombatAttributeValueKind::Rating,
                })
                .unwrap();
        }

        database
            .register_resource(ResourceDefinition {
                metadata: metadata("resource.mana", "Mana"),
            })
            .unwrap();

        database
            .register_damage_school(DamageSchoolDefinition {
                metadata: metadata("school.fire", "Fire"),
                mitigation,
            })
            .unwrap();

        database.validate().unwrap();

        database
    }

    fn resolved_effect(
        source: Option<Entity>,
        target: Entity,
        operation: ResolvedEffectOperation,
    ) -> ResolvedEffect {
        ResolvedEffect {
            effect: content_id("effect.test"),
            trigger: EffectTrigger::OnCast,
            source,
            target,
            operation,
        }
    }

    fn damage_effect(
        source: Entity,
        target: Entity,
    ) -> ResolvedEffect {
        resolved_effect(
            Some(source),
            target,
            ResolvedEffectOperation::Damage(
                super::super::DamageRequest {
                    source: Some(source),
                    target,
                    base_amount: 100.0,
                    school: content_id("school.fire"),
                },
            ),
        )
    }

    fn armor_id() -> ContentId {
        content_id("stat.armor")
    }

    fn fire_resistance_id() -> ContentId {
        content_id("stat.fire_resistance")
    }

    fn attributes_with_fire_resistance(
        resistance: f32,
    ) -> CombatAttributes {
        let mut attributes = CombatAttributes::default();

        attributes
            .set(fire_resistance_id(), resistance)
            .unwrap();

        attributes
    }

    fn execution_context<'a>(
        database: &'a ContentDatabase,
        attacker_level: Option<&'a CombatLevel>,
        target_attributes: Option<&'a CombatAttributes>,
        target_health: Option<&'a mut HealthPool>,
        target_resources: Option<&'a mut CombatResources>,
    ) -> EffectExecutionContext<'a> {
        EffectExecutionContext {
            database,
            attacker_level,
            target_attributes,
            target_health,
            target_resources,
        }
    }

    #[test]
    fn damage_execution_applies_resistance_mitigation() {
        let database = database_with_school(
            DamageSchoolMitigation::Resistance {
                stat: fire_resistance_id(),
            },
        );

        let source = Entity::from_bits(1);
        let target = Entity::from_bits(2);
        let attacker_level = CombatLevel::new(60).unwrap();
        let attributes = attributes_with_fire_resistance(200.0);
        let mut health = HealthPool::new(100.0, 100.0).unwrap();

        let result = execute_resolved_effect(
            &damage_effect(source, target),
            execution_context(
                &database,
                Some(&attacker_level),
                Some(&attributes),
                Some(&mut health),
                None,
            ),
        )
        .unwrap();

        assert_eq!(health.current(), 50.0);

        assert_eq!(
            result,
            EffectExecutionResult::Damage {
                resolution: DamageResolution::from_school_mitigation(
                    super::super::DamageRequest {
                        source: Some(source),
                        target,
                        base_amount: 100.0,
                        school: content_id("school.fire"),
                    },
                    database
                        .damage_school(&content_id("school.fire"))
                        .unwrap(),
                    &armor_id(),
                    &attributes,
                    &attacker_level,
                )
                .unwrap(),
                applied_amount: 50.0,
            }
        );
    }

    #[test]
    fn damage_execution_applies_armor_mitigation() {
        let database = database_with_school(
            DamageSchoolMitigation::Armor,
        );

        let source = Entity::from_bits(1);
        let target = Entity::from_bits(2);
        let attacker_level = CombatLevel::new(60).unwrap();

        let mut attributes = CombatAttributes::default();
        attributes
            .set(armor_id(), 5500.0)
            .unwrap();

        let mut health = HealthPool::new(100.0, 100.0).unwrap();

        execute_resolved_effect(
            &damage_effect(source, target),
            execution_context(
                &database,
                Some(&attacker_level),
                Some(&attributes),
                Some(&mut health),
                None,
            ),
        )
        .unwrap();

        assert_eq!(health.current(), 50.0);
    }

    #[test]
    fn healing_execution_caps_healing_at_remaining_health() {
        let database = database_with_school(
            DamageSchoolMitigation::Unmitigated,
        );

        let source = Entity::from_bits(1);
        let target = Entity::from_bits(2);

        let effect = resolved_effect(
            Some(source),
            target,
            ResolvedEffectOperation::Heal(
                super::super::HealingRequest {
                    source: Some(source),
                    target,
                    base_amount: 80.0,
                },
            ),
        );

        let mut health = HealthPool::new(50.0, 100.0).unwrap();

        let result = execute_resolved_effect(
            &effect,
            execution_context(
                &database,
                None,
                None,
                Some(&mut health),
                None,
            ),
        )
        .unwrap();

        assert_eq!(health.current(), 100.0);

        assert!(matches!(
            result,
            EffectExecutionResult::Healing {
                applied_amount: 50.0,
                ..
            }
        ));
    }

    #[test]
    fn resource_gain_execution_applies_effective_amount() {
        let database = database_with_school(
            DamageSchoolMitigation::Unmitigated,
        );

        let source = Entity::from_bits(1);
        let target = Entity::from_bits(2);

        let effect = resolved_effect(
            Some(source),
            target,
            ResolvedEffectOperation::GainResource(
                super::super::ResourceRequest {
                    source: Some(source),
                    target,
                    resource: content_id("resource.mana"),
                    operation: ResourceOperation::Gain,
                    base_amount: 25.0,
                },
            ),
        );

        let mut resources = CombatResources::default();

        resources.insert(
            content_id("resource.mana"),
            ResourcePool::new(10.0, 100.0).unwrap(),
        );

        let result = execute_resolved_effect(
            &effect,
            execution_context(
                &database,
                None,
                None,
                None,
                Some(&mut resources),
            ),
        )
        .unwrap();

        assert_eq!(
            resources.pool(&content_id("resource.mana")).unwrap().current(),
            35.0
        );

        assert!(matches!(
            result,
            EffectExecutionResult::Resource(resolution)
                if resolution.effective_amount == 25.0
        ));
    }

    #[test]
    fn resource_spending_execution_caps_at_available_amount() {
        let database = database_with_school(
            DamageSchoolMitigation::Unmitigated,
        );

        let source = Entity::from_bits(1);
        let target = Entity::from_bits(2);

        let effect = resolved_effect(
            Some(source),
            target,
            ResolvedEffectOperation::SpendResource(
                super::super::ResourceRequest {
                    source: Some(source),
                    target,
                    resource: content_id("resource.mana"),
                    operation: ResourceOperation::Spend,
                    base_amount: 30.0,
                },
            ),
        );

        let mut resources = CombatResources::default();

        resources.insert(
            content_id("resource.mana"),
            ResourcePool::new(10.0, 100.0).unwrap(),
        );

        let result = execute_resolved_effect(
            &effect,
            execution_context(
                &database,
                None,
                None,
                None,
                Some(&mut resources),
            ),
        )
        .unwrap();

        assert_eq!(
            resources.pool(&content_id("resource.mana")).unwrap().current(),
            0.0
        );

        assert!(matches!(
            result,
            EffectExecutionResult::Resource(resolution)
                if resolution.effective_amount == 10.0
        ));
    }

    #[test]
    fn resource_execution_rejects_mismatched_operation() {
        let database = database_with_school(
            DamageSchoolMitigation::Unmitigated,
        );

        let source = Entity::from_bits(1);
        let target = Entity::from_bits(2);

        let effect = resolved_effect(
            Some(source),
            target,
            ResolvedEffectOperation::GainResource(
                super::super::ResourceRequest {
                    source: Some(source),
                    target,
                    resource: content_id("resource.mana"),
                    operation: ResourceOperation::Spend,
                    base_amount: 30.0,
                },
            ),
        );

        let mut resources = CombatResources::default();

        resources.insert(
            content_id("resource.mana"),
            ResourcePool::new(50.0, 100.0).unwrap(),
        );

        let result = execute_resolved_effect(
            &effect,
            execution_context(
                &database,
                None,
                None,
                None,
                Some(&mut resources),
            ),
        );

        assert_eq!(
            result,
            Err(EffectExecutionError::ResourceOperationMismatch)
        );

        assert_eq!(
            resources.pool(&content_id("resource.mana")).unwrap().current(),
            50.0
        );
    }

    #[test]
    fn execution_rejects_mismatched_targets() {
        let database = database_with_school(
            DamageSchoolMitigation::Unmitigated,
        );

        let source = Entity::from_bits(1);
        let target = Entity::from_bits(2);

        let effect = damage_effect(source, target);

        let mut health = HealthPool::new(100.0, 100.0).unwrap();
        let attacker_level = CombatLevel::new(60).unwrap();
        let attributes = CombatAttributes::default();

        let mut mismatched_effect = effect.clone();

        mismatched_effect.target = Entity::from_bits(3);

        assert_eq!(
            execute_resolved_effect(
                &mismatched_effect,
                execution_context(
                    &database,
                    Some(&attacker_level),
                    Some(&attributes),
                    Some(&mut health),
                    None,
                ),
            ),
            Err(EffectExecutionError::ContextMismatch)
        );

        assert_eq!(health.current(), 100.0);
    }

    #[test]
    fn execution_rejects_unsupported_operations_without_mutation() {
        let database = database_with_school(
            DamageSchoolMitigation::Unmitigated,
        );

        let source = Entity::from_bits(1);
        let target = Entity::from_bits(2);

        let effect = resolved_effect(
            Some(source),
            target,
            ResolvedEffectOperation::ApplyAura {
                aura: content_id("aura.test"),
                stacks: 1,
                duration_seconds: Some(10.0),
            },
        );

        let mut health = HealthPool::new(50.0, 100.0).unwrap();

        assert_eq!(
            execute_resolved_effect(
                &effect,
                execution_context(
                    &database,
                    None,
                    None,
                    Some(&mut health),
                    None,
                ),
            ),
            Err(EffectExecutionError::UnsupportedOperation)
        );

        assert_eq!(health.current(), 50.0);
    }
}
