use super::*;

#[test]
fn initial_pitch_preserves_the_configured_camera_position() {
    let distance: f32 = 8.0;
    let height = 4.5;
    let vertical_offset = height - CAMERA_LOOK_HEIGHT;
    let boom_length = distance.hypot(vertical_offset);
    let pitch = -vertical_offset.atan2(distance);

    let offset = Quat::from_rotation_x(pitch) * Vec3::Z * boom_length;

    assert!((offset.y - vertical_offset).abs() < 1.0e-5);
    assert!((offset.z - distance).abs() < 1.0e-5);
}

#[test]
fn camera_pitch_limit_is_eighty_nine_degrees() {
    let pitch = CAMERA_PITCH_LIMIT + 1.0;
    let clamped = pitch.clamp(-CAMERA_PITCH_LIMIT, CAMERA_PITCH_LIMIT);

    assert_eq!(clamped, CAMERA_PITCH_LIMIT);
}

#[test]
fn zoom_moves_at_a_constant_speed() {
    let distance = advance_zoom_distance(8.0, 10.0, 0.1);

    assert!((distance - 8.833).abs() < 1.0e-5);
}

#[test]
fn zoom_does_not_overshoot_its_target() {
    let distance = advance_zoom_distance(8.0, 9.0, 1.0);

    assert_eq!(distance, 9.0);
}

#[test]
fn collision_pulls_the_camera_in_immediately() {
    let distance = advance_collision_distance(8.0, 3.0, 1.0 / 60.0);

    assert_eq!(distance, 3.0);
}

#[test]
fn collision_distance_returns_smoothly_when_clear() {
    let distance = advance_collision_distance(3.0, 8.0, 1.0 / 60.0);

    assert!(distance > 3.0);
    assert!(distance < 8.0);
}

#[test]
fn zoom_distance_is_bounded_by_the_requested_target() {
    let close = advance_zoom_distance(3.0, CAMERA_MIN_DISTANCE, 1.0);
    let far = advance_zoom_distance(49.0, CAMERA_MAX_DISTANCE, 1.0);

    assert!(close >= CAMERA_MIN_DISTANCE);
    assert!(far <= CAMERA_MAX_DISTANCE);
}

#[test]
fn camera_angle_uses_the_shortest_signed_representation() {
    let angle = wrap_camera_angle(5.5);

    assert!(angle < 0.0);
    assert!(angle > -std::f32::consts::PI);
}
