use super::*;

#[test]
fn follow_duration_scales_with_the_rotation() {
    let small = start_camera_follow(0.2).unwrap();
    let large = start_camera_follow(1.0).unwrap();

    assert!(large.duration > small.duration);
    assert!(small.duration >= CAMERA_FOLLOW_MIN_TIME);
    assert!(large.duration <= CAMERA_FOLLOW_MAX_TIME);
}

#[test]
fn follow_returns_to_the_character_with_smooth_easing() {
    let mut follow = start_camera_follow(1.0);
    let mut yaw = 1.0;

    advance_camera_follow(&mut follow, &mut yaw, 0.1);

    assert!(yaw < 1.0);
    assert!(yaw > 0.0);

    advance_camera_follow(&mut follow, &mut yaw, 5.0);

    assert_eq!(yaw, 0.0);
    assert!(follow.is_none());
}

#[test]
fn stopping_movement_cancels_the_follow_transition() {
    let mut orbit = CameraOrbitState {
        yaw_offset: 0.8,
        last_command: Some(CAMERA_CMD_FORWARD),
        follow_return: start_camera_follow(0.8),
        ..default()
    };

    update_camera_follow(
        &mut orbit,
        0,
        false,
        1.0 / 60.0,
    );

    assert!(orbit.follow_return.is_none());
    assert_eq!(orbit.yaw_offset, 0.8);
}

#[test]
fn mouse_look_cancels_the_follow_transition() {
    let mut orbit = CameraOrbitState {
        yaw_offset: 0.8,
        last_command: Some(CAMERA_CMD_FORWARD),
        follow_return: start_camera_follow(0.8),
        ..default()
    };

    update_camera_follow(
        &mut orbit,
        CAMERA_CMD_FORWARD | CAMERA_CMD_LEFT_MOUSE,
        true,
        1.0 / 60.0,
    );

    assert!(orbit.follow_return.is_none());
    assert_eq!(orbit.yaw_offset, 0.8);
}
