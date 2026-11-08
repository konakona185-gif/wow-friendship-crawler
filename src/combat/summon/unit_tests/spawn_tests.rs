use super::*;

#[test]
fn pending_summons_preserve_fifo_order() {
    let mut queue = PendingSummons::default();

    let first = PendingSummon {
        owner: Entity::from_bits(1),
        creature: creature_id(),
        spawn_position: Vec3::ZERO,
        count: 1,
        duration_seconds: None,
    };

    let second = PendingSummon {
        owner: Entity::from_bits(2),
        creature: creature_id(),
        spawn_position: Vec3::X,
        count: 2,
        duration_seconds: Some(10.0),
    };

    queue.enqueue(first.clone());
    queue.enqueue(second.clone());

    let pending = queue.take_pending();

    assert_eq!(pending, vec![first, second]);
    assert!(queue.is_empty());
}

#[test]
fn spawn_system_creates_ally_summons_at_owner_level() {
    let mut app = spawn_test_app();

    let owner = app
        .world_mut()
        .spawn((
            Character,
            CombatLevel::new(60).unwrap(),
            Transform::IDENTITY,
        ))
        .id();

    enqueue_spawn_request(&mut app, owner, 2);

    app.update();

    let world = app.world_mut();
    let mut query = world.query_filtered::<Entity, With<AllySummon>>();
    let entities: Vec<Entity> = query.iter(world).collect();

    assert_eq!(entities.len(), 2);

    for entity in entities {
        let summon = world.get::<SummonedCreature>(entity).unwrap();
        let level = world.get::<CombatLevel>(entity).unwrap();
        let health = world.get::<HealthPool>(entity).unwrap();
        let stats = world.get::<CombatStats>(entity).unwrap();
        let profile = world.get::<SummonCombatProfile>(entity).unwrap();
        let ai = world.get::<AllySummonAi>(entity).unwrap();
        let transform = world.get::<Transform>(entity).unwrap();

        assert_eq!(summon.owner, owner);
        assert_eq!(summon.faction, SummonFaction::Allies);
        assert_eq!(summon.creature, creature_id());
        assert_eq!(summon.remaining_duration(), Some(30.0));
        assert_eq!(level.value(), 60);
        assert_eq!(health.current(), 150.0);
        assert_eq!(health.maximum(), 150.0);
        assert_eq!(stats.current_health, 150.0);
        assert_eq!(stats.max_health, 150.0);
        assert_eq!(profile.attack_damage, 12.0);
        assert_eq!(profile.attack_interval_seconds, 2.0);
        assert_eq!(profile.melee_range, 2.5);
        assert_eq!(profile.aggro_range, 15.0);
        assert_eq!(profile.leash_range, 40.0);
        assert_eq!(profile.movement_speed, 5.0);
        assert_eq!(ai.target, None);
        assert_eq!(ai.aggro_range, 15.0);
        assert_eq!(
            transform.translation,
            Vec3::new(4.0, 2.0, -3.0)
        );
        assert!(world.get::<Enemy>(entity).is_none());
    }

    assert!(world.resource::<PendingSummons>().is_empty());
}

#[test]
fn spawn_system_creates_enemy_summons_with_enemy_components() {
    let mut app = spawn_test_app();

    let owner = app
        .world_mut()
        .spawn((
            Enemy,
            EnemyAi::default(),
            CombatLevel::new(42).unwrap(),
            Transform::IDENTITY,
        ))
        .id();

    enqueue_spawn_request(&mut app, owner, 1);

    app.update();

    let world = app.world_mut();
    let mut query = world.query_filtered::<Entity, With<Enemy>>();
    let entities: Vec<Entity> = query
        .iter(world)
        .filter(|entity| *entity != owner)
        .collect();

    assert_eq!(entities.len(), 1);

    let entity = entities[0];
    let summon = world.get::<SummonedCreature>(entity).unwrap();
    let level = world.get::<CombatLevel>(entity).unwrap();
    let ai = world.get::<EnemyAi>(entity).unwrap();
    let home = world.get::<EnemyHomePosition>(entity).unwrap();
    let profile = world.get::<SummonCombatProfile>(entity).unwrap();

    assert_eq!(profile.attack_damage, 12.0);
    assert_eq!(profile.attack_interval_seconds, 2.0);
    assert_eq!(profile.melee_range, 2.5);
    assert_eq!(profile.aggro_range, 15.0);
    assert_eq!(profile.leash_range, 40.0);
    assert_eq!(profile.movement_speed, 5.0);

    assert_eq!(summon.owner, owner);
    assert_eq!(summon.faction, SummonFaction::Enemies);
    assert_eq!(level.value(), 42);
    assert_eq!(ai.aggro_range, 15.0);
    assert_eq!(ai.leash_range, 40.0);
    assert_eq!(home.0, Vec3::new(4.0, 2.0, -3.0));
    assert!(world.get::<AllySummon>(entity).is_none());
    assert!(world.get::<AllySummonAi>(entity).is_none());
}

#[test]
fn spawned_summons_inherit_the_runtime_faction_of_their_owner() {
    let mut app = spawn_test_app();

    let owner = app
        .world_mut()
        .spawn((
            Enemy,
            EnemyAi::default(),
            SummonedCreature::new(
                Entity::from_bits(99),
                creature_id(),
                SummonFaction::Enemies,
                None,
            ),
            CombatLevel::new(60).unwrap(),
            Transform::IDENTITY,
        ))
        .id();

    enqueue_spawn_request(&mut app, owner, 1);

    app.update();

    let world = app.world_mut();
    let mut query = world.query_filtered::<Entity, With<SummonedCreature>>();
    let child = query
        .iter(world)
        .find(|entity| *entity != owner)
        .unwrap();

    let child_summon = world.get::<SummonedCreature>(child).unwrap();

    assert_eq!(child_summon.owner, owner);
    assert_eq!(child_summon.faction, SummonFaction::Enemies);
    assert!(world.get::<Enemy>(child).is_some());
    assert!(world.get::<AllySummon>(child).is_none());
    assert!(world.get::<AllySummonAi>(child).is_none());
}

#[test]
fn spawn_system_rejects_owner_without_runtime_faction() {
    let mut app = spawn_test_app();

    let owner = app
        .world_mut()
        .spawn((
            CombatLevel::new(60).unwrap(),
            Transform::IDENTITY,
        ))
        .id();

    enqueue_spawn_request(&mut app, owner, 1);

    app.update();

    let world = app.world_mut();
    let mut query = world.query::<&SummonedCreature>();

    assert_eq!(query.iter(world).count(), 0);
    assert!(world.resource::<PendingSummons>().is_empty());
}
