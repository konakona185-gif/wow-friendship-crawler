use std::time::Duration;

use bevy::prelude::*;

use super::super::{
    execute_pending_effects,
    PendingCombatEffects,
};

use crate::combat::{
    ActiveAuras,
    CombatAttributes,
    CombatEffectSubscriptions,
    CombatEventQueue,
    CombatLevel,
    CombatStats,
    EffectTargetSelection,
    Enemy,
    HealthPool,
};

use crate::content::{
    ContentDatabase,
    ContentId,
    ContentPackLoader,
};

fn corruption_apply_effect_id() -> ContentId {
    ContentId::new(
        "classic_1_12_1:effect.apply_corruption",
    )
    .unwrap()
}

fn corruption_tick_effect_id() -> ContentId {
    ContentId::new(
        "classic_1_12_1:effect.corruption_tick",
    )
    .unwrap()
}

fn corruption_aura_id() -> ContentId {
    ContentId::new(
        "classic_1_12_1:aura.corruption",
    )
    .unwrap()
}

fn corruption_runtime_app(
    database: ContentDatabase,
) -> App {
    let mut app = App::new();

    app.insert_resource(database)
        .insert_resource(Time::<()>::default())
        .init_resource::<PendingCombatEffects>()
        .init_resource::<CombatEventQueue>()
        .add_systems(
            Update,
            (
                crate::combat::aura::tick_active_auras,
                execute_pending_effects,
                crate::combat::effects::dispatch_combat_events,
            )
                .chain(),
        );

    app
}

fn spawn_corruption_caster(app: &mut App) -> Entity {
    let mut subscriptions =
        CombatEffectSubscriptions::default();

    subscriptions.subscribe(corruption_tick_effect_id());

    app.world_mut()
        .spawn((
            CombatLevel::new(60).unwrap(),
            CombatAttributes::default(),
            subscriptions,
        ))
        .id()
}

fn setup_corruption_scenario() -> (
    App,
    Entity,
    Entity,
) {
    let mut database = ContentDatabase::new();

    ContentPackLoader::load_ron(
        &mut database,
        include_str!("../../../../../content/packs/classic_1_12_1.ron"),
    )
    .unwrap();

    let mut app = corruption_runtime_app(database);

    let caster = spawn_corruption_caster(&mut app);

    let target = app
        .world_mut()
        .spawn((
            Enemy,
            Transform::default(),
            CombatAttributes::default(),
            CombatStats {
                current_health: 1000.0,
                max_health: 1000.0,
                resource: 0.0,
            },
            HealthPool::new(1000.0, 1000.0).unwrap(),
        ))
        .id();

    (app, caster, target)
}

fn enqueue_corruption(
    app: &mut App,
    caster: Entity,
    target: Entity,
) {
    let database = app
        .world()
        .resource::<ContentDatabase>()
        .clone();

    let source_attributes = app
        .world()
        .get::<CombatAttributes>(caster)
        .unwrap()
        .clone();

    let resolved_count = app
        .world_mut()
        .resource_mut::<PendingCombatEffects>()
        .resolve_and_enqueue_from_database(
            &database,
            &corruption_apply_effect_id(),
            Some(caster),
            EffectTargetSelection::Selected(target),
            &source_attributes,
        )
        .unwrap();

    assert_eq!(resolved_count, 1);
}

fn advance_corruption_tick(
    app: &mut App,
    target: Entity,
    previous_health: f32,
    expected_health: f32,
) {
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs(3));

    app.update();

    assert_eq!(
        app.world()
            .get::<HealthPool>(target)
            .unwrap()
            .current(),
        previous_health
    );

    assert_eq!(
        app.world()
            .resource::<PendingCombatEffects>()
            .len(),
        1
    );

    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::ZERO);

    app.update();

    assert_eq!(
        app.world()
            .get::<HealthPool>(target)
            .unwrap()
            .current(),
        expected_health
    );

    assert_eq!(
        app.world()
            .get::<CombatStats>(target)
            .unwrap()
            .current_health,
        expected_health
    );

    assert!(
        app.world()
            .resource::<PendingCombatEffects>()
            .is_empty()
    );

    assert!(
        app.world()
            .resource::<CombatEventQueue>()
            .is_empty()
    );
}

#[test]
fn corruption_from_classic_pack_executes_periodic_shadow_damage_end_to_end() {
    let (mut app, caster, target) =
        setup_corruption_scenario();

    enqueue_corruption(&mut app, caster, target);

    app.update();

    let active_auras = app
        .world()
        .get::<ActiveAuras>(target)
        .unwrap();

    assert_eq!(
        active_auras.stacks(&corruption_aura_id()),
        1
    );

    let active_aura = active_auras
        .get(&corruption_aura_id())
        .unwrap();

    assert_eq!(active_aura.source(), Some(caster));
    assert_eq!(
        active_aura.remaining_duration(),
        Some(18.0)
    );

    assert!(
        app.world()
            .resource::<PendingCombatEffects>()
            .is_empty()
    );

    advance_corruption_tick(
        &mut app,
        target,
        1000.0,
        863.0,
    );

    let active_aura = app
        .world()
        .get::<ActiveAuras>(target)
        .unwrap()
        .get(&corruption_aura_id())
        .unwrap();

    assert_eq!(
        active_aura.remaining_duration(),
        Some(15.0)
    );
}

#[test]
fn corruption_repeats_periodic_ticks_and_preserves_pandemic_carryover() {
    let (mut app, caster, target) =
        setup_corruption_scenario();

    enqueue_corruption(&mut app, caster, target);

    app.update();

    advance_corruption_tick(
        &mut app,
        target,
        1000.0,
        863.0,
    );

    let active_aura = app
        .world()
        .get::<ActiveAuras>(target)
        .unwrap()
        .get(&corruption_aura_id())
        .unwrap();

    assert_eq!(
        active_aura.remaining_duration(),
        Some(15.0)
    );

    advance_corruption_tick(
        &mut app,
        target,
        863.0,
        726.0,
    );

    let active_aura = app
        .world()
        .get::<ActiveAuras>(target)
        .unwrap()
        .get(&corruption_aura_id())
        .unwrap();

    assert_eq!(
        active_aura.remaining_duration(),
        Some(12.0)
    );

    enqueue_corruption(&mut app, caster, target);

    app.update();

    let active_auras = app
        .world()
        .get::<ActiveAuras>(target)
        .unwrap();

    assert_eq!(
        active_auras.len(),
        1
    );

    assert_eq!(
        active_auras.stacks(&corruption_aura_id()),
        1
    );

    let active_aura = active_auras
        .get(&corruption_aura_id())
        .unwrap();

    assert_eq!(active_aura.source(), Some(caster));
    assert_eq!(
        active_aura.remaining_duration(),
        Some(23.4)
    );

    advance_corruption_tick(
        &mut app,
        target,
        726.0,
        589.0,
    );

    let active_aura = app
        .world()
        .get::<ActiveAuras>(target)
        .unwrap()
        .get(&corruption_aura_id())
        .unwrap();

    assert_eq!(
        active_aura.remaining_duration(),
        Some(20.4)
    );

    assert_eq!(
        active_aura.stacks(),
        1
    );
}

#[test]
fn corruption_from_two_casters_stacks_independently_and_ticks_both_sources() {
    let (mut app, first_caster, target) =
        setup_corruption_scenario();

    let second_caster = spawn_corruption_caster(&mut app);

    enqueue_corruption(&mut app, first_caster, target);
    enqueue_corruption(&mut app, second_caster, target);

    app.update();

    let active_auras = app
        .world()
        .get::<ActiveAuras>(target)
        .unwrap();

    assert_eq!(active_auras.len(), 2);
    assert_eq!(
        active_auras.stacks(&corruption_aura_id()),
        2
    );

    assert_eq!(
        active_auras
            .iter()
            .filter(|aura| aura.source() == Some(first_caster))
            .count(),
        1
    );

    assert_eq!(
        active_auras
            .iter()
            .filter(|aura| aura.source() == Some(second_caster))
            .count(),
        1
    );

    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs(3));

    app.update();

    assert_eq!(
        app.world()
            .get::<HealthPool>(target)
            .unwrap()
            .current(),
        1000.0
    );

    assert_eq!(
        app.world()
            .resource::<PendingCombatEffects>()
            .len(),
        2
    );

    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::ZERO);

    app.update();

    assert_eq!(
        app.world()
            .get::<HealthPool>(target)
            .unwrap()
            .current(),
        726.0
    );

    assert_eq!(
        app.world()
            .get::<CombatStats>(target)
            .unwrap()
            .current_health,
        726.0
    );

    assert!(
        app.world()
            .resource::<PendingCombatEffects>()
            .is_empty()
    );

    assert!(
        app.world()
            .resource::<CombatEventQueue>()
            .is_empty()
    );
}

#[test]
fn corruption_executes_final_tick_at_expiration_and_stops_afterward() {
    let (mut app, caster, target) =
        setup_corruption_scenario();

    enqueue_corruption(&mut app, caster, target);

    app.update();

    let expected_health_values = [
        863.0,
        726.0,
        589.0,
        452.0,
        315.0,
        178.0,
    ];

    let mut previous_health = 1000.0;

    for expected_health in expected_health_values {
        advance_corruption_tick(
            &mut app,
            target,
            previous_health,
            expected_health,
        );

        previous_health = expected_health;
    }

    assert!(
        app.world()
            .get::<ActiveAuras>(target)
            .unwrap()
            .is_empty()
    );

    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs(3));

    app.update();

    assert_eq!(
        app.world()
            .get::<HealthPool>(target)
            .unwrap()
            .current(),
        178.0
    );

    assert!(
        app.world()
            .resource::<PendingCombatEffects>()
            .is_empty()
    );

    assert!(
        app.world()
            .resource::<CombatEventQueue>()
            .is_empty()
    );
}
