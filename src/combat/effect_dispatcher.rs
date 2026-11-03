use bevy::prelude::*;

use crate::content::{
    ContentDatabase,
    EffectTarget,
    EffectTrigger,
};

use super::{
    CombatAttributes,
    CombatEffectSubscriptions,
    CombatEventContext,
    CombatEventQueue,
    EffectTargetSelection,
    PendingCombatEffects,
    MAX_REACTIVE_EFFECT_GENERATIONS,
};

fn event_participants(
    event: &CombatEventContext,
) -> Option<(Entity, Option<Entity>)> {
    match event.trigger {
        EffectTrigger::OnDamageDealt | EffectTrigger::OnHealDealt => {
            Some((event.source?, event.target))
        }
        EffectTrigger::OnDamageTaken
        | EffectTrigger::OnHealTaken
        | EffectTrigger::OnResourceGained
        | EffectTrigger::OnResourceSpent => {
            Some((event.target?, event.source))
        }
        _ => None,
    }
}

pub(super) fn dispatch_combat_events(
    mut event_queue: ResMut<CombatEventQueue>,
    database: Res<ContentDatabase>,
    mut pending: ResMut<PendingCombatEffects>,
    subscribers: Query<(
        &CombatEffectSubscriptions,
        Option<&CombatAttributes>,
    )>,
) {
    let events = event_queue.drain_with_generation();

    for (event, generation) in events {
        if generation >= MAX_REACTIVE_EFFECT_GENERATIONS {
            continue;
        }

        let Some((subscriber, other_participant)) =
            event_participants(&event)
        else {
            continue;
        };

        let Ok((subscriptions, attributes)) = subscribers.get(subscriber)
        else {
            continue;
        };

        let next_generation = generation + 1;
        let source_attributes =
            attributes.cloned().unwrap_or_default();

        for effect_id in subscriptions.iter() {
            let Some(definition) = database.effect(effect_id) else {
                warn!(
                    "Effetto reattivo non registrato: {}",
                    effect_id.as_str()
                );
                continue;
            };

            if definition.trigger != event.trigger {
                continue;
            }

            let target_selection = match &definition.target {
                EffectTarget::Caster => EffectTargetSelection::Caster,
                EffectTarget::SelectedTarget => {
                    let Some(target) = other_participant else {
                        continue;
                    };

                    EffectTargetSelection::Selected(target)
                }
                EffectTarget::AlliesInRadius { .. }
                | EffectTarget::EnemiesInRadius { .. }
                | EffectTarget::UnitsInRadius { .. } => {
                    warn!(
                        "Effetto reattivo ad area non ancora supportato: {}",
                        effect_id.as_str()
                    );
                    continue;
                }
            };

            if let Err(error) =
                pending.resolve_and_enqueue_from_database_with_generation(
                    &database,
                    effect_id,
                    Some(subscriber),
                    target_selection,
                    &source_attributes,
                    next_generation,
                )
            {
                warn!(
                    "Impossibile accodare l'effetto reattivo {}: {}",
                    effect_id.as_str(),
                    error
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::content::{
        ContentId,
        ContentMetadata,
        ContentPackManifest,
        EffectDefinition,
        EffectKind,
        EffectMagnitude,
    };

    fn source() -> Entity {
        Entity::from_bits(1)
    }

    fn target() -> Entity {
        Entity::from_bits(2)
    }

    fn event(
        trigger: EffectTrigger,
        source: Option<Entity>,
        target: Option<Entity>,
    ) -> CombatEventContext {
        CombatEventContext {
            trigger,
            source,
            target,
            ability: None,
            effect: None,
            outcome: None,
        }
    }

    fn test_application(
        trigger: EffectTrigger,
        target: EffectTarget,
    ) -> (App, ContentId) {
        let pack_id =
            ContentId::new("pack:dispatcher_tests").unwrap();

        let effect_id =
            ContentId::new("dispatcher_tests:effect.reactive").unwrap();

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
                metadata: ContentMetadata {
                    id: effect_id.clone(),
                    pack_id,
                    name: "Reactive Test".to_string(),
                    description: String::new(),
                    icon: None,
                    tags: Vec::new(),
                },
                trigger,
                target,
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
            .add_systems(Update, dispatch_combat_events);

        (app, effect_id)
    }

    fn spawn_subscriber(
        app: &mut App,
        effect_id: ContentId,
    ) -> Entity {
        let mut subscriptions =
            CombatEffectSubscriptions::default();

        subscriptions.subscribe(effect_id);

        app.world_mut()
            .spawn((
                subscriptions,
                CombatAttributes::default(),
            ))
            .id()
    }

    #[test]
    fn damage_and_healing_dealt_events_use_source_as_subscriber() {
        for trigger in [
            EffectTrigger::OnDamageDealt,
            EffectTrigger::OnHealDealt,
        ] {
            assert_eq!(
                event_participants(&event(
                    trigger,
                    Some(source()),
                    Some(target()),
                )),
                Some((source(), Some(target())))
            );
        }
    }

    #[test]
    fn damage_and_healing_taken_events_use_target_as_subscriber() {
        for trigger in [
            EffectTrigger::OnDamageTaken,
            EffectTrigger::OnHealTaken,
        ] {
            assert_eq!(
                event_participants(&event(
                    trigger,
                    Some(source()),
                    Some(target()),
                )),
                Some((target(), Some(source())))
            );
        }
    }

    #[test]
    fn resource_events_use_recipient_as_subscriber() {
        for trigger in [
            EffectTrigger::OnResourceGained,
            EffectTrigger::OnResourceSpent,
        ] {
            assert_eq!(
                event_participants(&event(
                    trigger,
                    Some(source()),
                    Some(target()),
                )),
                Some((target(), Some(source())))
            );
        }
    }

    #[test]
    fn unsupported_triggers_do_not_select_subscribers() {
        for trigger in [
            EffectTrigger::OnCast,
            EffectTrigger::OnHit,
            EffectTrigger::OnCriticalHit,
            EffectTrigger::OnPeriodicTick,
            EffectTrigger::OnKill,
            EffectTrigger::OnDodge,
            EffectTrigger::OnBlock,
            EffectTrigger::OnExpire,
        ] {
            assert_eq!(
                event_participants(&event(
                    trigger,
                    Some(source()),
                    Some(target()),
                )),
                None
            );
        }
    }

    #[test]
    fn missing_required_participants_skip_dispatch() {
        assert_eq!(
            event_participants(&event(
                EffectTrigger::OnDamageDealt,
                None,
                Some(target()),
            )),
            None
        );

        assert_eq!(
            event_participants(&event(
                EffectTrigger::OnDamageTaken,
                Some(source()),
                None,
            )),
            None
        );
    }

    #[test]
    fn damage_dealt_event_enqueues_matching_reactive_effect() {
        let (mut app, effect_id) = test_application(
            EffectTrigger::OnDamageDealt,
            EffectTarget::SelectedTarget,
        );

        let subscriber = spawn_subscriber(&mut app, effect_id);
        let other = app.world_mut().spawn_empty().id();

        app.world_mut()
            .resource_mut::<CombatEventQueue>()
            .enqueue(event(
                EffectTrigger::OnDamageDealt,
                Some(subscriber),
                Some(other),
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
    fn mismatched_event_trigger_does_not_enqueue_effect() {
        let (mut app, effect_id) = test_application(
            EffectTrigger::OnHealDealt,
            EffectTarget::SelectedTarget,
        );

        let subscriber = spawn_subscriber(&mut app, effect_id);
        let other = app.world_mut().spawn_empty().id();

        app.world_mut()
            .resource_mut::<CombatEventQueue>()
            .enqueue(event(
                EffectTrigger::OnDamageDealt,
                Some(subscriber),
                Some(other),
            ))
            .unwrap();

        app.update();

        assert_eq!(
            app.world().resource::<PendingCombatEffects>().len(),
            0
        );
    }

    #[test]
    fn maximum_generation_does_not_enqueue_reactive_effects() {
        let (mut app, effect_id) = test_application(
            EffectTrigger::OnDamageDealt,
            EffectTarget::SelectedTarget,
        );

        let subscriber = spawn_subscriber(&mut app, effect_id);
        let other = app.world_mut().spawn_empty().id();

        app.world_mut()
            .resource_mut::<CombatEventQueue>()
            .enqueue_with_generation(
                event(
                    EffectTrigger::OnDamageDealt,
                    Some(subscriber),
                    Some(other),
                ),
                MAX_REACTIVE_EFFECT_GENERATIONS,
            )
            .unwrap();

        app.update();

        assert_eq!(
            app.world().resource::<PendingCombatEffects>().len(),
            0
        );
    }
}
