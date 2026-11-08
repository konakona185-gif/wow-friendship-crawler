use super::*;

#[test]
fn effect_subscriptions_start_empty() {
    let subscriptions = CombatEffectSubscriptions::default();

    assert!(subscriptions.is_empty());
    assert_eq!(subscriptions.len(), 0);
}

#[test]
fn effect_subscriptions_reject_duplicates() {
    let mut subscriptions = CombatEffectSubscriptions::default();
    let effect = content_id("effect.passive");

    assert!(subscriptions.subscribe(effect.clone()));
    assert!(!subscriptions.subscribe(effect.clone()));

    assert_eq!(subscriptions.len(), 1);
    assert!(subscriptions.contains(&effect));
}

#[test]
fn effect_subscriptions_preserve_registration_order() {
    let mut subscriptions = CombatEffectSubscriptions::default();

    let first = content_id("effect.first");
    let second = content_id("effect.second");

    assert!(subscriptions.subscribe(first.clone()));
    assert!(subscriptions.subscribe(second.clone()));

    assert_eq!(
        subscriptions.iter().cloned().collect::<Vec<_>>(),
        vec![first, second]
    );
}

#[test]
fn effect_subscriptions_remove_only_requested_effect() {
    let mut subscriptions = CombatEffectSubscriptions::default();

    let first = content_id("effect.first");
    let second = content_id("effect.second");

    subscriptions.subscribe(first.clone());
    subscriptions.subscribe(second.clone());

    assert!(subscriptions.unsubscribe(&first));
    assert!(!subscriptions.unsubscribe(&first));

    assert_eq!(subscriptions.len(), 1);
    assert!(!subscriptions.contains(&first));
    assert!(subscriptions.contains(&second));
}
