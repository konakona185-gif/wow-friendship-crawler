use super::*;

#[test]
fn self_fade_hides_the_character_at_first_person_distance() {
    let alpha = self_model_fade_alpha(
        0.0,
        0.1,
        SELF_FADE_WINDOW,
    );

    assert_eq!(alpha, 0.0);
}

#[test]
fn self_fade_restores_the_character_at_normal_distance() {
    let alpha = self_model_fade_alpha(
        0.1 + SELF_FADE_WINDOW,
        0.1,
        SELF_FADE_WINDOW,
    );

    assert_eq!(alpha, 1.0);
}

#[test]
fn self_fade_is_continuous_between_first_and_third_person() {
    let alpha = self_model_fade_alpha(
        0.1 + SELF_FADE_WINDOW * 0.5,
        0.1,
        SELF_FADE_WINDOW,
    );

    assert!((alpha - 0.5).abs() < 1.0e-5);
}
