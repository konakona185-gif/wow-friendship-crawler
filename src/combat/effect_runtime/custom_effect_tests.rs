use super::*;

use crate::combat::{
    CombatAttributes,
    CombatEventQueue,
    CustomEffectHandler,
    CustomEffectHandlers,
    EffectExecutionContext,
    EffectExecutionError,
    EffectTargetSelection,
    HealthPool,
};
use crate::content::{
    ContentMetadata,
    ContentPackManifest,
    EffectDefinition,
    EffectKind,
    EffectTarget,
    EffectTrigger,
};

struct HealingCustomEffectHandler;

impl CustomEffectHandler for HealingCustomEffectHandler {
    fn execute(
        &self,
        _effect: &ResolvedEffect,
        context: &mut EffectExecutionContext<'_>,
    ) -> Result<(), EffectExecutionError> {
        let target_health = context
            .target_health
            .as_deref_mut()
            .ok_or(EffectExecutionError::MissingTargetHealth)?;

        target_health
            .apply_healing(15.0)
            .map_err(EffectExecutionError::HealthApplication)?;

        Ok(())
    }
}

fn content_id(value: &str) -> ContentId {
    ContentId::new(
        format!("classic_1_12_1:{value}"),
    )
    .unwrap()
}

fn handler_id() -> ContentId {
    content_id("handler.custom_heal")
}

fn custom_test_database() -> (ContentDatabase, ContentId) {
    let mut database = ContentDatabase::new();

    let pack_id = ContentId::new(
        "pack:classic_1_12_1",
    )
    .unwrap();

    database
        .register_pack(ContentPackManifest {
            id: pack_id.clone(),
            name: "Classic 1.12.1".to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            dependencies: Vec::new(),
        })
        .unwrap();

    let effect_id = content_id("effect.custom_heal");

    database
        .register_effect(EffectDefinition {
            metadata: ContentMetadata {
                id: effect_id.clone(),
                pack_id,
                name: "Custom Heal".to_string(),
                description: String::new(),
                icon: None,
                tags: Vec::new(),
            },
            trigger: EffectTrigger::OnCast,
            target: EffectTarget::SelectedTarget,
            kind: EffectKind::Custom {
                handler: handler_id(),
            },
        })
        .unwrap();

    database.validate().unwrap();

    (database, effect_id)
}

fn app_with_custom_effect(
    custom_handlers: CustomEffectHandlers,
) -> (App, Entity) {
    let (database, effect_id) = custom_test_database();

    let mut app = App::new();

    let source_entity = app
        .world_mut()
        .spawn_empty()
        .id();

    let target_entity = app
        .world_mut()
        .spawn(
            HealthPool::new(40.0, 100.0).unwrap(),
        )
        .id();

    let mut pending = PendingCombatEffects::default();

    let count = pending
        .resolve_and_enqueue_from_database(
            &database,
            &effect_id,
            Some(source_entity),
            EffectTargetSelection::Selected(target_entity),
            &CombatAttributes::default(),
        )
        .unwrap();

    assert_eq!(count, 1);

    app.insert_resource(database)
        .insert_resource(pending)
        .insert_resource(custom_handlers)
        .init_resource::<CombatEventQueue>()
        .add_systems(Update, execute_pending_effects);

    app.update();

    (app, target_entity)
}

#[test]
fn runtime_executes_registered_custom_effect_handler() {
    let mut handlers = CustomEffectHandlers::default();

    handlers
        .register(
            handler_id(),
            HealingCustomEffectHandler,
        )
        .unwrap();

    let (app, target_entity) = app_with_custom_effect(handlers);

    let world = app.world();

    let health = world
        .get::<HealthPool>(target_entity)
        .unwrap();

    assert_eq!(health.current(), 55.0);
    assert_eq!(health.maximum(), 100.0);

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
fn runtime_ignores_custom_effect_without_registered_handler() {
    let handlers = CustomEffectHandlers::default();

    let (app, target_entity) = app_with_custom_effect(handlers);

    let world = app.world();

    let health = world
        .get::<HealthPool>(target_entity)
        .unwrap();

    assert_eq!(health.current(), 40.0);
    assert_eq!(health.maximum(), 100.0);

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
