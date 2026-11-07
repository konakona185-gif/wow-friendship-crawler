use std::time::Duration;

use bevy::prelude::*;

use super::{
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
                crate::combat::aura_systems::tick_active_auras,
                execute_pending_effects,
                crate::combat::effect_dispatcher::dispatch_combat_events,
            )
                .chain(),
        );

    app
}

#[test]
fn corruption_from_classic_pack_executes_periodic_shadow_damage_end_to_end() {
    let mut database = ContentDatabase::new();

    ContentPackLoader::load_ron(
        &mut database,
        include_str!("../../../content/packs/classic_1_12_1.ron"),
    )
    .unwrap();

    let mut app = corruption_runtime_app(database);

    let mut subscriptions =
        CombatEffectSubscriptions::default();

    subscriptions.subscribe(corruption_tick_effect_id());

    let caster = app
        .world_mut()
        .spawn((
            CombatLevel::new(60).unwrap(),
            CombatAttributes::default(),
            subscriptions,
        ))
        .id();

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
        863.0
    );

    assert_eq!(
        app.world()
            .get::<CombatStats>(target)
            .unwrap()
            .current_health,
        863.0
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
