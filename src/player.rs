use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;

#[derive(Component)]
pub struct Character;

#[derive(Component)]
pub enum FellowshipRole {
    Tank,
    Healer,
    Dps,
}

#[derive(Component)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

#[derive(Component)]
pub struct MovementSpeed(pub f32);

#[derive(Component)]
pub struct Velocity(pub Vec3);

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, player_movement_system);
    }
}

fn player_movement_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &MovementSpeed, &mut Velocity), With<Character>>,
) {
    let gravity = -25.0;
    let jump_force = 9.0;
    let floor_y = 1.0;

    for (mut transform, speed, mut velocity) in &mut query {
        if mouse_button.pressed(MouseButton::Right) {
            let mut mouse_delta = 0.0;
            for event in mouse_motion.read() {
                mouse_delta += event.delta.x;
            }
            
            if mouse_delta != 0.0 {
                let sensitivity = 0.003;
                transform.rotate_y(-mouse_delta * sensitivity);
            }
        } else {
            mouse_motion.clear();
        }

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

        let mut move_direction = Vec3::ZERO;
        if forward_move != 0.0 || strafe_move != 0.0 {
            let forward = transform.forward().to_array();
            let right = transform.right().to_array();
            
            let forward_vec = Vec3::from_array(forward);
            let right_vec = Vec3::from_array(right);
            
            move_direction = (forward_vec * forward_move) + (right_vec * strafe_move);
            move_direction.y = 0.0;
            
            if move_direction != Vec3::ZERO {
                move_direction = move_direction.normalize();
            }
        }

        let is_grounded = transform.translation.y <= floor_y;

        if is_grounded {
            velocity.0.y = 0.0;
            transform.translation.y = floor_y;

            if keyboard_input.just_pressed(KeyCode::Space) {
                velocity.0.y = jump_force;
            }
        } else {
            velocity.0.y += gravity * time.delta_seconds();
        }

        let horizontal_velocity = move_direction * speed.0;
        transform.translation += horizontal_velocity * time.delta_seconds();
        transform.translation.y += velocity.0.y * time.delta_seconds();

        if transform.translation.y < floor_y {
            transform.translation.y = floor_y;
            velocity.0.y = 0.0;
        }
    }
}
