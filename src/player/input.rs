use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;

use super::{CharacterCommand, LocalPlayer};

pub(super) fn local_player_command_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mut query: Query<&mut CharacterCommand, With<LocalPlayer>>,
) {
    let mut forward_move = 0.0;
    let mut strafe_move = 0.0;

    if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
        forward_move += 1.0;
    }

    if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
        forward_move -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
        strafe_move -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
        strafe_move += 1.0;
    }

    let mouse_delta = if mouse_button.pressed(MouseButton::Right) {
        mouse_motion.delta.x
    } else {
        0.0
    };

    for mut command in &mut query {
        command.movement = Vec2::new(strafe_move, forward_move);
        command.jump = keyboard_input.just_pressed(KeyCode::Space);
        command.look_delta = mouse_delta;
        command.select_target = keyboard_input.just_pressed(KeyCode::Tab);
        command.ability_1 = keyboard_input.just_pressed(KeyCode::Digit1);
    }
}
