use super::*;

#[test]
fn reactive_area_effect_acquires_enemies_from_world() {
    let (mut app, effect_id) = test_application(
        EffectTrigger::OnDamageDealt,
        EffectTarget::EnemiesInRadius { radius: 5.0 },
    );

    let mut subscriptions =
        CombatEffectSubscriptions::default();

    subscriptions.subscribe(effect_id);

    let subscriber = app.world_mut()
        .spawn((
            subscriptions,
            CombatAttributes::default(),
            Character,
            Transform::from_translation(Vec3::ZERO),
        ))
        .id();

    let ally = app.world_mut()
        .spawn((
            Character,
            Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
        ))
        .id();

    app.world_mut()
        .spawn((
            Enemy,
            Transform::from_translation(Vec3::new(3.0, 0.0, 0.0)),
        ));

    app.world_mut()
        .spawn((
            Enemy,
            Transform::from_translation(Vec3::new(8.0, 0.0, 0.0)),
        ));

    app.world_mut()
        .resource_mut::<CombatEventQueue>()
        .enqueue(event(
            EffectTrigger::OnDamageDealt,
            Some(subscriber),
            Some(ally),
        ))
        .unwrap();

    app.update();

    assert_eq!(
        app.world().resource::<PendingCombatEffects>().len(),
        1
    );

    assert!(
        app.world().resource::<CombatEventQueue>().is_empty()
    );
}

#[test]
fn reactive_area_effect_executes_only_on_enemies_inside_radius() {
    let pack_id =
        ContentId::new("pack:dispatcher_tests").unwrap();

    let effect_id = content_id("effect.area_reactive");

    let mut database = ContentDatabase::new();

    database
        .register_pack(ContentPackManifest {
            id: pack_id.clone(),
            name: "Dispatcher Tests".to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            dependencies: Vec::new(),
        })
        .unwrap();

    database
        .register_effect(EffectDefinition {
            metadata: metadata(
                effect_id.clone(),
                &pack_id,
                "Area Reactive Heal",
            ),
            trigger: EffectTrigger::OnDamageDealt,
            target: EffectTarget::EnemiesInRadius {
                radius: 5.0,
            },
            kind: EffectKind::Heal {
                amount: EffectMagnitude {
                    base: 10.0,
                    scaling: Vec::new(),
                },
            },
        })
        .unwrap();

    let mut app = App::new();

    app.insert_resource(database)
        .init_resource::<CombatEventQueue>()
        .init_resource::<PendingCombatEffects>()
        .add_systems(
            Update,
            (
                super::super::super::runtime::execute_pending_effects,
                dispatch_combat_events,
            )
                .chain(),
        );

    let mut subscriptions =
        CombatEffectSubscriptions::default();

    subscriptions.subscribe(effect_id);

    let subscriber = app.world_mut()
        .spawn((
            subscriptions,
            CombatAttributes::default(),
            Character,
            Transform::from_translation(Vec3::ZERO),
        ))
        .id();

    let ally = app.world_mut()
        .spawn((
            Character,
            Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
        ))
        .id();

    let nearby_enemy = app.world_mut()
        .spawn((
            Enemy,
            Transform::from_translation(Vec3::new(3.0, 0.0, 0.0)),
            CombatStats {
                current_health: 50.0,
                max_health: 100.0,
                resource: 0.0,
            },
            HealthPool::new(50.0, 100.0).unwrap(),
        ))
        .id();

    let distant_enemy = app.world_mut()
        .spawn((
            Enemy,
            Transform::from_translation(Vec3::new(8.0, 0.0, 0.0)),
            CombatStats {
                current_health: 50.0,
                max_health: 100.0,
                resource: 0.0,
            },
            HealthPool::new(50.0, 100.0).unwrap(),
        ))
        .id();

    app.world_mut()
        .resource_mut::<CombatEventQueue>()
        .enqueue(event(
            EffectTrigger::OnDamageDealt,
            Some(subscriber),
            Some(ally),
        ))
        .unwrap();

    app.update();

    assert_eq!(
        app.world()
            .get::<HealthPool>(nearby_enemy)
            .unwrap()
            .current(),
        50.0
    );

    assert_eq!(
        app.world()
            .resource::<PendingCombatEffects>()
            .len(),
        1
    );

    app.update();

    assert_eq!(
        app.world()
            .get::<HealthPool>(nearby_enemy)
            .unwrap()
            .current(),
        60.0
    );

    assert_eq!(
        app.world()
            .get::<HealthPool>(distant_enemy)
            .unwrap()
            .current(),
        50.0
    );

    assert_eq!(
        app.world()
            .resource::<PendingCombatEffects>()
            .len(),
        0
    );
}

#[derive(Clone, Copy)]
enum TestCombatantSide {
    Character,
    AllySummon,
    Enemy,
}

fn spawn_area_combatant(
    app: &mut App,
    side: TestCombatantSide,
    position: Vec3,
    current_health: f32,
) -> Entity {
    let mut entity = app.world_mut().spawn((
        Transform::from_translation(position),
        CombatStats {
            current_health,
            max_health: 100.0,
            resource: 0.0,
        },
        HealthPool::new(current_health, 100.0).unwrap(),
    ));

    match side {
        TestCombatantSide::Character => {
            entity.insert(Character);
        }
        TestCombatantSide::AllySummon => {
            entity.insert(AllySummon);
        }
        TestCombatantSide::Enemy => {
            entity.insert(Enemy);
        }
    }

    entity.id()
}

#[test]
fn reactive_area_effects_from_ally_summon_respect_both_factions_end_to_end() {
    let pack_id =
        ContentId::new("pack:dispatcher_tests").unwrap();

    let ally_effect_id = content_id("effect.area_allies");
    let enemy_effect_id = content_id("effect.area_enemies");

    let mut database = ContentDatabase::new();

    database
        .register_pack(ContentPackManifest {
            id: pack_id.clone(),
            name: "Dispatcher Tests".to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            dependencies: Vec::new(),
        })
        .unwrap();

    database
        .register_effect(EffectDefinition {
            metadata: metadata(
                ally_effect_id.clone(),
                &pack_id,
                "Area Allies Heal",
            ),
            trigger: EffectTrigger::OnDamageDealt,
            target: EffectTarget::AlliesInRadius {
                radius: 5.0,
            },
            kind: EffectKind::Heal {
                amount: EffectMagnitude {
                    base: 10.0,
                    scaling: Vec::new(),
                },
            },
        })
        .unwrap();

    database
        .register_effect(EffectDefinition {
            metadata: metadata(
                enemy_effect_id.clone(),
                &pack_id,
                "Area Enemies Heal",
            ),
            trigger: EffectTrigger::OnDamageDealt,
            target: EffectTarget::EnemiesInRadius {
                radius: 5.0,
            },
            kind: EffectKind::Heal {
                amount: EffectMagnitude {
                    base: 20.0,
                    scaling: Vec::new(),
                },
            },
        })
        .unwrap();

    let mut app = App::new();

    app.insert_resource(database)
        .init_resource::<CombatEventQueue>()
        .init_resource::<PendingCombatEffects>()
        .add_systems(
            Update,
            (
                super::super::super::runtime::execute_pending_effects,
                dispatch_combat_events,
            )
                .chain(),
        );

    let mut subscriptions =
        CombatEffectSubscriptions::default();

    subscriptions.subscribe(ally_effect_id);
    subscriptions.subscribe(enemy_effect_id);

    let source_summon = spawn_area_combatant(
        &mut app,
        TestCombatantSide::AllySummon,
        Vec3::ZERO,
        40.0,
    );

    app.world_mut()
        .entity_mut(source_summon)
        .insert((
            subscriptions,
            CombatAttributes::default(),
        ));

    let nearby_character = spawn_area_combatant(
        &mut app,
        TestCombatantSide::Character,
        Vec3::new(1.0, 0.0, 0.0),
        40.0,
    );

    let nearby_ally_summon = spawn_area_combatant(
        &mut app,
        TestCombatantSide::AllySummon,
        Vec3::new(2.0, 0.0, 0.0),
        30.0,
    );

    let nearby_enemy = spawn_area_combatant(
        &mut app,
        TestCombatantSide::Enemy,
        Vec3::new(3.0, 0.0, 0.0),
        80.0,
    );

    let distant_character = spawn_area_combatant(
        &mut app,
        TestCombatantSide::Character,
        Vec3::new(8.0, 0.0, 0.0),
        40.0,
    );

    let distant_enemy = spawn_area_combatant(
        &mut app,
        TestCombatantSide::Enemy,
        Vec3::new(9.0, 0.0, 0.0),
        60.0,
    );

    app.world_mut()
        .resource_mut::<CombatEventQueue>()
        .enqueue(event(
            EffectTrigger::OnDamageDealt,
            Some(source_summon),
            Some(nearby_enemy),
        ))
        .unwrap();

    app.update();

    assert_eq!(
        app.world()
            .resource::<PendingCombatEffects>()
            .len(),
        4,
        "Gli effetti ad area devono accodare tre bersagli alleati e un nemico."
    );

    app.update();

    assert_eq!(
        app.world()
            .get::<HealthPool>(source_summon)
            .unwrap()
            .current(),
        50.0
    );

    assert_eq!(
        app.world()
            .get::<HealthPool>(nearby_character)
            .unwrap()
            .current(),
        50.0
    );

    assert_eq!(
        app.world()
            .get::<HealthPool>(nearby_ally_summon)
            .unwrap()
            .current(),
        40.0
    );

    assert_eq!(
        app.world()
            .get::<HealthPool>(nearby_enemy)
            .unwrap()
            .current(),
        100.0
    );

    assert_eq!(
        app.world()
            .get::<HealthPool>(distant_character)
            .unwrap()
            .current(),
        40.0
    );

    assert_eq!(
        app.world()
            .get::<HealthPool>(distant_enemy)
            .unwrap()
            .current(),
        60.0
    );

    assert!(
        app.world()
            .resource::<PendingCombatEffects>()
            .is_empty()
    );
}

#[test]
fn periodic_area_effect_scales_each_enemy_by_stack_count() {
    let pack_id =
        ContentId::new("pack:dispatcher_tests").unwrap();

    let effect_id = content_id("effect.periodic_area");

    let mut database = ContentDatabase::new();

    database
        .register_pack(ContentPackManifest {
            id: pack_id.clone(),
            name: "Dispatcher Tests".to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            dependencies: Vec::new(),
        })
        .unwrap();

    database
        .register_effect(EffectDefinition {
            metadata: metadata(
                effect_id.clone(),
                &pack_id,
                "Periodic Area Heal",
            ),
            trigger: EffectTrigger::OnPeriodicTick,
            target: EffectTarget::EnemiesInRadius {
                radius: 5.0,
            },
            kind: EffectKind::Heal {
                amount: EffectMagnitude {
                    base: 10.0,
                    scaling: Vec::new(),
                },
            },
        })
        .unwrap();

    let mut app = App::new();

    app.insert_resource(database)
        .init_resource::<CombatEventQueue>()
        .init_resource::<PendingCombatEffects>()
        .add_systems(
            Update,
            (
                super::super::super::runtime::execute_pending_effects,
                dispatch_combat_events,
            )
                .chain(),
        );

    let mut subscriptions =
        CombatEffectSubscriptions::default();

    subscriptions.subscribe(effect_id.clone());

    let subscriber = spawn_area_combatant(
        &mut app,
        TestCombatantSide::Character,
        Vec3::ZERO,
        50.0,
    );

    app.world_mut()
        .entity_mut(subscriber)
        .insert((
            subscriptions,
            CombatAttributes::default(),
        ));

    let first_nearby_enemy = spawn_area_combatant(
        &mut app,
        TestCombatantSide::Enemy,
        Vec3::new(3.0, 0.0, 0.0),
        50.0,
    );

    let second_nearby_enemy = spawn_area_combatant(
        &mut app,
        TestCombatantSide::Enemy,
        Vec3::new(4.0, 0.0, 0.0),
        40.0,
    );

    let distant_enemy = spawn_area_combatant(
        &mut app,
        TestCombatantSide::Enemy,
        Vec3::new(8.0, 0.0, 0.0),
        50.0,
    );

    app.world_mut()
        .resource_mut::<CombatEventQueue>()
        .enqueue(CombatEventContext {
            trigger: EffectTrigger::OnPeriodicTick,
            source: Some(subscriber),
            target: Some(first_nearby_enemy),
            ability: None,
            effect: Some(effect_id),
            outcome: Some(periodic_tick_outcome(
                3,
                PeriodicEffectStackScaling::Linear,
            )),
        })
        .unwrap();

    app.update();

    assert_eq!(
        app.world()
            .resource::<PendingCombatEffects>()
            .len(),
        2
    );

    assert_eq!(
        app.world()
            .get::<HealthPool>(first_nearby_enemy)
            .unwrap()
            .current(),
        50.0
    );

    assert_eq!(
        app.world()
            .get::<HealthPool>(second_nearby_enemy)
            .unwrap()
            .current(),
        40.0
    );

    app.update();

    assert_eq!(
        app.world()
            .get::<HealthPool>(first_nearby_enemy)
            .unwrap()
            .current(),
        80.0
    );

    assert_eq!(
        app.world()
            .get::<HealthPool>(second_nearby_enemy)
            .unwrap()
            .current(),
        70.0
    );

    assert_eq!(
        app.world()
            .get::<HealthPool>(distant_enemy)
            .unwrap()
            .current(),
        50.0
    );

    assert!(
        app.world()
            .resource::<PendingCombatEffects>()
            .is_empty()
    );
}

#[test]
fn periodic_area_targeting_uses_health_pool_as_authoritative_alive_state() {
    let pack_id =
        ContentId::new("pack:dispatcher_tests").unwrap();

    let effect_id = content_id("effect.periodic_health_pool");

    let mut database = ContentDatabase::new();

    database
        .register_pack(ContentPackManifest {
            id: pack_id.clone(),
            name: "Dispatcher Tests".to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            dependencies: Vec::new(),
        })
        .unwrap();

    database
        .register_effect(EffectDefinition {
            metadata: metadata(
                effect_id.clone(),
                &pack_id,
                "Periodic Health Pool Test",
            ),
            trigger: EffectTrigger::OnPeriodicTick,
            target: EffectTarget::EnemiesInRadius {
                radius: 5.0,
            },
            kind: EffectKind::Heal {
                amount: EffectMagnitude {
                    base: 10.0,
                    scaling: Vec::new(),
                },
            },
        })
        .unwrap();

    let mut app = App::new();

    app.insert_resource(database)
        .init_resource::<CombatEventQueue>()
        .init_resource::<PendingCombatEffects>()
        .add_systems(
            Update,
            (
                super::super::super::runtime::execute_pending_effects,
                dispatch_combat_events,
            )
                .chain(),
        );

    let mut subscriptions =
        CombatEffectSubscriptions::default();

    subscriptions.subscribe(effect_id.clone());

    let subscriber = spawn_area_combatant(
        &mut app,
        TestCombatantSide::Character,
        Vec3::ZERO,
        50.0,
    );

    app.world_mut()
        .entity_mut(subscriber)
        .insert((
            subscriptions,
            CombatAttributes::default(),
        ));

    let living_enemy = spawn_area_combatant(
        &mut app,
        TestCombatantSide::Enemy,
        Vec3::new(3.0, 0.0, 0.0),
        50.0,
    );

    app.world_mut()
        .get_mut::<CombatStats>(living_enemy)
        .unwrap()
        .current_health = 0.0;

    let defeated_enemy = spawn_area_combatant(
        &mut app,
        TestCombatantSide::Enemy,
        Vec3::new(4.0, 0.0, 0.0),
        50.0,
    );

    app.world_mut()
        .get_mut::<HealthPool>(defeated_enemy)
        .unwrap()
        .set_current(0.0)
        .unwrap();

    app.world_mut()
        .resource_mut::<CombatEventQueue>()
        .enqueue(CombatEventContext {
            trigger: EffectTrigger::OnPeriodicTick,
            source: Some(subscriber),
            target: Some(living_enemy),
            ability: None,
            effect: Some(effect_id),
            outcome: Some(periodic_tick_outcome(
                1,
                PeriodicEffectStackScaling::None,
            )),
        })
        .unwrap();

    app.update();

    assert_eq!(
        app.world()
            .resource::<PendingCombatEffects>()
            .len(),
        1
    );

    assert_eq!(
        app.world()
            .get::<HealthPool>(living_enemy)
            .unwrap()
            .current(),
        50.0
    );

    assert_eq!(
        app.world()
            .get::<HealthPool>(defeated_enemy)
            .unwrap()
            .current(),
        0.0
    );

    app.update();

    assert_eq!(
        app.world()
            .get::<HealthPool>(living_enemy)
            .unwrap()
            .current(),
        60.0
    );

    assert_eq!(
        app.world()
            .get::<HealthPool>(defeated_enemy)
            .unwrap()
            .current(),
        0.0
    );

    assert_eq!(
        app.world()
            .resource::<PendingCombatEffects>()
            .len(),
        0
    );
}
