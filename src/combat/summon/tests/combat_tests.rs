use super::*;

#[test]
fn allied_summon_attacks_generate_threat_against_the_enemy() {
    let mut app = setup_summon_integration_app();

    let owner = app
        .world_mut()
        .spawn((
            Character,
            CombatLevel::new(60).unwrap(),
            Transform::IDENTITY,
            CombatStats {
                current_health: 100.0,
                max_health: 100.0,
                resource: 0.0,
            },
            HealthPool::new(100.0, 100.0).unwrap(),
        ))
        .id();

    let enemy = app
        .world_mut()
        .spawn((
            Enemy,
            EnemyAi::default(),
            CombatLevel::new(60).unwrap(),
            Transform::from_xyz(0.0, 0.0, 2.0),
            CombatStats {
                current_health: 100.0,
                max_health: 100.0,
                resource: 0.0,
            },
            HealthPool::new(100.0, 100.0).unwrap(),
        ))
        .id();

    enqueue_summon_effect(&mut app, owner);

    app.update();

    let summon_entities = {
        let world = app.world_mut();
        let mut summon_query = world.query_filtered::<Entity, With<AllySummon>>();

        summon_query.iter(world).collect::<Vec<_>>()
    };

    assert_eq!(summon_entities.len(), 2);

    app.update();

    advance_test_time(&mut app, 2.0);
    app.update();

    let threat = app.world().get::<EnemyThreat>(enemy).unwrap();

    assert_eq!(threat.values.len(), summon_entities.len());

    for summon in &summon_entities {
        assert!(threat.values.get(summon).is_some_and(|value| *value > 0.0));
    }

    assert!(app.world().get::<HealthPool>(enemy).unwrap().current() < 100.0);

    app.update();

    let enemy_target = app.world().get::<EnemyAi>(enemy).unwrap().target;

    assert!(enemy_target.is_some_and(|target| { summon_entities.contains(&target) }));
}

#[test]
fn enemy_summons_acquire_and_attack_a_player_end_to_end() {
    let mut app = setup_summon_integration_app();

    let owner = app
        .world_mut()
        .spawn((
            Enemy,
            EnemyAi {
                returning_home: true,
                ..Default::default()
            },
            CombatLevel::new(60).unwrap(),
            Transform::IDENTITY,
        ))
        .id();

    let player = app
        .world_mut()
        .spawn((
            Character,
            CombatLevel::new(60).unwrap(),
            Transform::from_xyz(0.0, 0.0, 2.0),
            CombatStats {
                current_health: 100.0,
                max_health: 100.0,
                resource: 0.0,
            },
            HealthPool::new(100.0, 100.0).unwrap(),
        ))
        .id();

    enqueue_summon_effect(&mut app, owner);

    app.update();

    let summon_entities = {
        let world = app.world_mut();
        let mut summon_query =
            world.query_filtered::<Entity, (With<Enemy>, With<SummonedCreature>)>();

        summon_query.iter(world).collect::<Vec<_>>()
    };

    assert_eq!(summon_entities.len(), 2);

    for summon in &summon_entities {
        let ai = app.world().get::<EnemyAi>(*summon).unwrap();

        assert_eq!(ai.target, Some(player));
        assert!(app.world().get::<AllySummon>(*summon).is_none());
    }

    advance_test_time(&mut app, 2.0);
    app.update();

    let player_health = app.world().get::<HealthPool>(player).unwrap();
    let player_stats = app.world().get::<CombatStats>(player).unwrap();

    assert_eq!(player_health.current(), 76.0);
    assert_eq!(player_stats.current_health, 76.0);
}
