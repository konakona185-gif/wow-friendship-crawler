use super::*;

#[test]
fn damage_and_healing_dealt_events_use_source_as_subscriber() {
    for trigger in [EffectTrigger::OnDamageDealt, EffectTrigger::OnHealDealt] {
        assert_eq!(
            event_participants(&event(trigger, Some(source()), Some(target()),)),
            Some((source(), Some(target())))
        );
    }
}

#[test]
fn damage_and_healing_taken_events_use_target_as_subscriber() {
    for trigger in [EffectTrigger::OnDamageTaken, EffectTrigger::OnHealTaken] {
        assert_eq!(
            event_participants(&event(trigger, Some(source()), Some(target()),)),
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
            event_participants(&event(trigger, Some(source()), Some(target()),)),
            Some((target(), Some(source())))
        );
    }
}

#[test]
fn periodic_tick_events_use_source_as_subscriber() {
    assert_eq!(
        event_participants(&event(
            EffectTrigger::OnPeriodicTick,
            Some(source()),
            Some(target()),
        )),
        Some((source(), Some(target())))
    );

    assert_eq!(
        event_participants(&event(EffectTrigger::OnPeriodicTick, None, Some(target()),)),
        None
    );
}

#[test]
fn periodic_tick_events_match_only_their_declared_effect() {
    let matching_effect = content_id("effect.periodic");
    let other_effect = content_id("effect.other");

    let mut matching_event = event(
        EffectTrigger::OnPeriodicTick,
        Some(source()),
        Some(target()),
    );

    matching_event.effect = Some(matching_effect.clone());

    let mut other_event = matching_event.clone();
    other_event.effect = Some(other_effect);

    assert!(event_matches_effect(&matching_event, &matching_effect));

    assert!(!event_matches_effect(&other_event, &matching_effect));

    let event_without_effect = event(
        EffectTrigger::OnPeriodicTick,
        Some(source()),
        Some(target()),
    );

    assert!(!event_matches_effect(
        &event_without_effect,
        &matching_effect
    ));
}

#[test]
fn aura_expiration_events_use_owner_as_subscriber() {
    assert_eq!(
        event_participants(&event(
            EffectTrigger::OnExpire,
            Some(source()),
            Some(target()),
        )),
        Some((target(), Some(source())))
    );

    assert_eq!(
        event_participants(&event(EffectTrigger::OnExpire, None, Some(target()),)),
        Some((target(), None))
    );

    assert_eq!(
        event_participants(&event(EffectTrigger::OnExpire, Some(source()), None,)),
        None
    );
}

#[test]
fn unsupported_triggers_do_not_select_subscribers() {
    for trigger in [
        EffectTrigger::OnCast,
        EffectTrigger::OnHit,
        EffectTrigger::OnCriticalHit,
        EffectTrigger::OnKill,
        EffectTrigger::OnDodge,
        EffectTrigger::OnBlock,
    ] {
        assert_eq!(
            event_participants(&event(trigger, Some(source()), Some(target()),)),
            None
        );
    }
}

#[test]
fn missing_required_participants_skip_dispatch() {
    assert_eq!(
        event_participants(&event(EffectTrigger::OnDamageDealt, None, Some(target()),)),
        None
    );

    assert_eq!(
        event_participants(&event(EffectTrigger::OnDamageTaken, Some(source()), None,)),
        None
    );
}
