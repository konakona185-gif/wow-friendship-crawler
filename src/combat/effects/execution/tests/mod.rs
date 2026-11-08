    use super::*;

    use bevy::prelude::Entity;

    use crate::combat::{
        CombatAttributes,
        CombatLevel,
        CombatResources,
        EffectTrigger,
        EnemyThreat,
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
                crate::combat::DamageRequest {
                    source: Some(source),
                    target,
                    base_amount: 100.0,
                    school: content_id("school.fire"),
                },
            ),
        )
    }

    fn threat_effect(
        source: Option<Entity>,
        target: Entity,
        amount: f32,
    ) -> ResolvedEffect {
        resolved_effect(
            source,
            target,
            ResolvedEffectOperation::ModifyThreat {
                amount,
            },
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
            target_threat: None,
        }
    }

    fn execution_context_with_threat<'a>(
        database: &'a ContentDatabase,
        attacker_level: Option<&'a CombatLevel>,
        target_attributes: Option<&'a CombatAttributes>,
        target_health: Option<&'a mut HealthPool>,
        target_resources: Option<&'a mut CombatResources>,
        target_threat: Option<&'a mut EnemyThreat>,
    ) -> EffectExecutionContext<'a> {
        EffectExecutionContext {
            database,
            attacker_level,
            target_attributes,
            target_health,
            target_resources,
            target_threat,
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
                    crate::combat::DamageRequest {
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
                crate::combat::HealingRequest {
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
                crate::combat::ResourceRequest {
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
                crate::combat::ResourceRequest {
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
                crate::combat::ResourceRequest {
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

    #[test]
    fn threat_execution_adds_threat_for_effect_source() {
        let database = database_with_school(
            DamageSchoolMitigation::Unmitigated,
        );

        let source = Entity::from_bits(1);
        let target = Entity::from_bits(2);
        let effect = threat_effect(
            Some(source),
            target,
            40.0,
        );

        let mut threat = EnemyThreat::default();

        let result = execute_resolved_effect(
            &effect,
            execution_context_with_threat(
                &database,
                None,
                None,
                None,
                None,
                Some(&mut threat),
            ),
        )
        .unwrap();

        assert_eq!(
            threat.values.get(&source),
            Some(&40.0)
        );

        assert_eq!(
            result,
            EffectExecutionResult::Threat {
                modified_amount: 40.0,
            }
        );
    }

    #[test]
    fn threat_execution_reduces_existing_threat_for_effect_source() {
        let database = database_with_school(
            DamageSchoolMitigation::Unmitigated,
        );

        let source = Entity::from_bits(1);
        let target = Entity::from_bits(2);
        let effect = threat_effect(
            Some(source),
            target,
            -25.0,
        );

        let mut threat = EnemyThreat::default();
        threat.values.insert(source, 60.0);

        execute_resolved_effect(
            &effect,
            execution_context_with_threat(
                &database,
                None,
                None,
                None,
                None,
                Some(&mut threat),
            ),
        )
        .unwrap();

        assert_eq!(
            threat.values.get(&source),
            Some(&35.0)
        );
    }

    #[test]
    fn threat_execution_removes_entry_when_threat_reaches_zero() {
        let database = database_with_school(
            DamageSchoolMitigation::Unmitigated,
        );

        let source = Entity::from_bits(1);
        let target = Entity::from_bits(2);
        let effect = threat_effect(
            Some(source),
            target,
            -50.0,
        );

        let mut threat = EnemyThreat::default();
        threat.values.insert(source, 30.0);

        execute_resolved_effect(
            &effect,
            execution_context_with_threat(
                &database,
                None,
                None,
                None,
                None,
                Some(&mut threat),
            ),
        )
        .unwrap();

        assert!(
            !threat.values.contains_key(&source)
        );
    }

    #[test]
    fn threat_execution_requires_source() {
        let database = database_with_school(
            DamageSchoolMitigation::Unmitigated,
        );

        let target = Entity::from_bits(2);
        let effect = threat_effect(
            None,
            target,
            40.0,
        );

        let mut threat = EnemyThreat::default();

        let result = execute_resolved_effect(
            &effect,
            execution_context_with_threat(
                &database,
                None,
                None,
                None,
                None,
                Some(&mut threat),
            ),
        );

        assert_eq!(
            result,
            Err(
                EffectExecutionError::MissingSourceForThreatModification
            )
        );

        assert!(threat.values.is_empty());
    }

    #[test]
    fn threat_execution_requires_target_threat_component() {
        let database = database_with_school(
            DamageSchoolMitigation::Unmitigated,
        );

        let source = Entity::from_bits(1);
        let target = Entity::from_bits(2);
        let effect = threat_effect(
            Some(source),
            target,
            40.0,
        );

        let result = execute_resolved_effect(
            &effect,
            execution_context(
                &database,
                None,
                None,
                None,
                None,
            ),
        );

        assert_eq!(
            result,
            Err(EffectExecutionError::MissingTargetThreat)
        );
    }

    #[test]
    fn threat_execution_rejects_non_finite_amounts() {
        let database = database_with_school(
            DamageSchoolMitigation::Unmitigated,
        );

        let source = Entity::from_bits(1);
        let target = Entity::from_bits(2);
        let effect = threat_effect(
            Some(source),
            target,
            f32::NAN,
        );

        let mut threat = EnemyThreat::default();

        let result = execute_resolved_effect(
            &effect,
            execution_context_with_threat(
                &database,
                None,
                None,
                None,
                None,
                Some(&mut threat),
            ),
        );

        assert_eq!(
            result,
            Err(EffectExecutionError::InvalidThreatAmount)
        );

        assert!(threat.values.is_empty());
    }
