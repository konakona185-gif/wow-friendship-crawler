use super::*;

#[test]
fn event_queue_starts_empty() {
    let queue = CombatEventQueue::default();

    assert!(queue.is_empty());
    assert_eq!(queue.len(), 0);
}

#[test]
fn event_queue_preserves_fifo_order() {
    let mut queue = CombatEventQueue::default();

    let mut first = valid_context(EffectTrigger::OnCast);
    first.effect = Some(content_id("effect.first"));

    let mut second = valid_context(EffectTrigger::OnDamageDealt);
    second.effect = Some(content_id("effect.second"));

    queue.enqueue(first.clone()).unwrap();
    queue.enqueue(second.clone()).unwrap();

    assert_eq!(queue.len(), 2);

    let drained = queue.drain();

    assert_eq!(drained, vec![first, second]);
    assert!(queue.is_empty());
}

#[test]
fn event_queue_preserves_generation_metadata() {
    let mut queue = CombatEventQueue::default();

    let first = valid_context(EffectTrigger::OnCast);
    let second = valid_context(EffectTrigger::OnDamageDealt);

    queue
        .enqueue_with_generation(first.clone(), 0)
        .unwrap();
    queue
        .enqueue_with_generation(second.clone(), 7)
        .unwrap();

    let drained = queue.drain_with_generation();

    assert_eq!(
        drained,
        vec![
            (first, 0),
            (second, 7),
        ]
    );
    assert!(queue.is_empty());
}

#[test]
fn event_queue_uses_generation_zero_for_standard_enqueue() {
    let mut queue = CombatEventQueue::default();
    let event = valid_context(EffectTrigger::OnCast);

    queue.enqueue(event.clone()).unwrap();

    assert_eq!(
        queue.drain_with_generation(),
        vec![(event, 0)]
    );
}

#[test]
fn event_queue_rejects_invalid_events_without_mutation() {
    let mut queue = CombatEventQueue::default();

    let mut invalid = valid_context(EffectTrigger::OnDamageDealt);
    invalid.source = None;
    invalid.outcome = Some(damage_outcome());

    assert_eq!(
        queue.enqueue(invalid),
        Err(CombatEventContextError::SourceMismatch)
    );

    assert!(queue.is_empty());
}
