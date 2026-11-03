use bevy::prelude::*;

use super::{
    ActiveAuras,
    CombatEventContext,
    CombatEventOutcome,
    CombatEventQueue,
    EffectTrigger,
};

pub(super) fn tick_active_auras(
    time: Res<Time>,
    mut event_queue: ResMut<CombatEventQueue>,
    mut aura_owners: Query<(Entity, &mut ActiveAuras)>,
) {
    let delta_seconds = time.delta_secs();

    for (owner, mut active_auras) in &mut aura_owners {
        let expired_auras = active_auras.tick(delta_seconds);

        for expired in expired_auras {
            let event = CombatEventContext {
                trigger: EffectTrigger::OnExpire,
                source: expired.source,
                target: Some(owner),
                ability: None,
                effect: None,
                outcome: Some(CombatEventOutcome::AuraExpired {
                    aura: expired.aura.clone(),
                    stacks: expired.stacks,
                }),
            };

            if let Err(error) = event_queue.enqueue(event) {
                warn!(
                    owner = ?owner,
                    aura = ?expired.aura,
                    %error,
                    "Impossibile registrare l'evento di scadenza dell'aura."
                );
                continue;
            }

            debug!(
                owner = ?owner,
                aura = ?expired.aura,
                stacks = expired.stacks,
                "Aura scaduta."
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::time::Duration;

    use crate::combat::ActiveAuras;
    use crate::content::{
        AuraDefinition,
        AuraStackingPolicy,
        ContentId,
        ContentMetadata,
    };

    fn aura_definition() -> AuraDefinition {
        AuraDefinition {
            metadata: ContentMetadata {
                id: ContentId::new(
                    "classic_1_12_1:aura.test",
                )
                .unwrap(),
                pack_id: ContentId::new(
                    "pack:classic_1_12_1",
                )
                .unwrap(),
                name: "Test Aura".to_string(),
                description: String::new(),
                icon: None,
                tags: Vec::new(),
            },
            max_stacks: 5,
            stacking_policy: AuraStackingPolicy::AddStacks,
            periodic_effects: Vec::new(),
        }
    }

    fn aura_id() -> ContentId {
        ContentId::new(
            "classic_1_12_1:aura.test",
        )
        .unwrap()
    }

    fn active_auras(duration: Option<f32>) -> ActiveAuras {
        let mut active = ActiveAuras::default();

        active
            .apply(
                &aura_definition(),
                None,
                1,
                duration,
            )
            .unwrap();

        active
    }

    fn app_with_delta(delta: Duration) -> App {
        let mut app = App::new();

        let mut time = Time::default();
        time.advance_by(delta);

        app.insert_resource(time)
            .init_resource::<CombatEventQueue>()
            .add_systems(Update, tick_active_auras);

        app
    }

    #[test]
    fn aura_duration_decreases_during_updates() {
        let mut app = app_with_delta(Duration::from_secs(2));

        let entity = app
            .world_mut()
            .spawn(active_auras(Some(5.0)))
            .id();

        app.update();

        let active = app
            .world()
            .get::<ActiveAuras>(entity)
            .unwrap();

        assert_eq!(
            active
                .get(&aura_id())
                .unwrap()
                .remaining_duration(),
            Some(3.0)
        );

        assert!(
            app.world()
                .resource::<CombatEventQueue>()
                .is_empty()
        );
    }

    #[test]
    fn expired_auras_are_removed_during_updates() {
        let mut app = app_with_delta(Duration::from_secs(6));

        let entity = app
            .world_mut()
            .spawn(active_auras(Some(5.0)))
            .id();

        app.update();

        let active = app
            .world()
            .get::<ActiveAuras>(entity)
            .unwrap();

        assert!(active.is_empty());

        let events = app
            .world_mut()
            .resource_mut::<CombatEventQueue>()
            .drain();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].trigger, EffectTrigger::OnExpire);
        assert_eq!(events[0].source, None);
        assert_eq!(events[0].target, Some(entity));

        assert_eq!(
            events[0].outcome,
            Some(CombatEventOutcome::AuraExpired {
                aura: aura_id(),
                stacks: 1,
            })
        );
    }

    #[test]
    fn permanent_auras_survive_updates() {
        let mut app = app_with_delta(Duration::from_secs(60));

        let entity = app
            .world_mut()
            .spawn(active_auras(None))
            .id();

        app.update();

        let active = app
            .world()
            .get::<ActiveAuras>(entity)
            .unwrap();

        assert_eq!(active.stacks(&aura_id()), 1);
        assert_eq!(
            active
                .get(&aura_id())
                .unwrap()
                .remaining_duration(),
            None
        );

        assert!(
            app.world()
                .resource::<CombatEventQueue>()
                .is_empty()
        );
    }

    #[test]
    fn each_entity_has_independent_aura_durations() {
        let mut app = app_with_delta(Duration::from_secs(5));

        let first = app
            .world_mut()
            .spawn(active_auras(Some(5.0)))
            .id();

        let second = app
            .world_mut()
            .spawn(active_auras(Some(10.0)))
            .id();

        app.update();

        let first_auras = app
            .world()
            .get::<ActiveAuras>(first)
            .unwrap();

        let second_auras = app
            .world()
            .get::<ActiveAuras>(second)
            .unwrap();

        assert!(first_auras.is_empty());

        assert_eq!(
            second_auras
                .get(&aura_id())
                .unwrap()
                .remaining_duration(),
            Some(5.0)
        );

        let events = app
            .world_mut()
            .resource_mut::<CombatEventQueue>()
            .drain();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].target, Some(first));
    }
}
