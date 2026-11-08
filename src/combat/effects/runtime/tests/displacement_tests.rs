
use bevy::prelude::{
    App,
    Entity,
    Transform,
    Update,
    Vec3,
};

use super::super::{
    execute_pending_effects,
    PendingCombatEffects,
    ResolvedEffect,
    ResolvedEffectOperation,
};

use crate::combat::{
    CombatEventQueue,
    PendingDisplacement,
};
use crate::content::{
    ContentDatabase,
    ContentId,
    EffectTrigger,
};

fn content_id(value: &str) -> ContentId {
    ContentId::new(
        format!("classic_1_12_1:{value}"),
    )
    .unwrap()
}

fn displacement_app(
    source_position: Option<Vec3>,
    target_position: Vec3,
    distances: &[f32],
    include_pending_component: bool,
) -> (App, Entity) {
    let mut app = App::new();

    let source_entity = source_position.map(|position| {
        app.world_mut()
            .spawn(Transform::from_translation(position))
            .id()
    });

    let target_entity = if include_pending_component {
        app.world_mut()
            .spawn((
                Transform::from_translation(target_position),
                PendingDisplacement::default(),
            ))
            .id()
    } else {
        app.world_mut()
            .spawn(
                Transform::from_translation(target_position),
            )
            .id()
    };

    let mut pending = PendingCombatEffects::default();

    for distance in distances {
        pending.enqueue(ResolvedEffect {
            effect: content_id("effect.displacement_test"),
            trigger: EffectTrigger::OnCast,
            source: source_entity,
            target: target_entity,
            operation: ResolvedEffectOperation::Displacement {
                distance: *distance,
            },
        });
    }

    app.insert_resource(ContentDatabase::new())
        .insert_resource(pending)
        .init_resource::<CombatEventQueue>()
        .add_systems(Update, execute_pending_effects);

    (app, target_entity)
}

#[test]
fn runtime_accumulates_displacements_without_changing_transform() {
    let (mut app, target_entity) = displacement_app(
        Some(Vec3::ZERO),
        Vec3::X * 5.0,
        &[4.0, 2.0],
        true,
    );

    app.update();

    let world = app.world();

    let displacement = world
        .get::<PendingDisplacement>(target_entity)
        .unwrap();

    assert_eq!(
        displacement.pending_translation(),
        Vec3::X * 6.0
    );

    let transform = world
        .get::<Transform>(target_entity)
        .unwrap();

    assert_eq!(
        transform.translation,
        Vec3::X * 5.0
    );

    assert!(
        world
            .resource::<PendingCombatEffects>()
            .is_empty()
    );

    assert!(
        world
            .resource::<CombatEventQueue>()
            .is_empty()
    );
}

#[test]
fn runtime_ignores_displacement_without_source() {
    let (mut app, target_entity) = displacement_app(
        None,
        Vec3::X * 5.0,
        &[3.0],
        true,
    );

    app.update();

    let world = app.world();

    let displacement = world
        .get::<PendingDisplacement>(target_entity)
        .unwrap();

    assert!(
        displacement.is_empty()
    );

    let transform = world
        .get::<Transform>(target_entity)
        .unwrap();

    assert_eq!(
        transform.translation,
        Vec3::X * 5.0
    );
}

#[test]
fn runtime_ignores_displacement_when_source_and_target_overlap() {
    let (mut app, target_entity) = displacement_app(
        Some(Vec3::new(5.0, 0.0, 0.0)),
        Vec3::new(5.0, 0.0, 0.0),
        &[3.0],
        true,
    );

    app.update();

    let world = app.world();

    let displacement = world
        .get::<PendingDisplacement>(target_entity)
        .unwrap();

    assert!(
        displacement.is_empty()
    );
}

#[test]
fn runtime_ignores_displacement_without_pending_component() {
    let (mut app, target_entity) = displacement_app(
        Some(Vec3::ZERO),
        Vec3::X * 5.0,
        &[3.0],
        false,
    );

    app.update();

    let world = app.world();

    assert!(
        world
            .get::<PendingDisplacement>(target_entity)
            .is_none()
    );

    let transform = world
        .get::<Transform>(target_entity)
        .unwrap();

    assert_eq!(
        transform.translation,
        Vec3::X * 5.0
    );
}
