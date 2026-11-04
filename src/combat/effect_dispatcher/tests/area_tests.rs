
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
                super::super::effect_runtime::execute_pending_effects,
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
