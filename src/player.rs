use bevy::prelude::*;

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

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, player_movement_system);
    }
}

fn player_movement_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &MovementSpeed), With<Character>>,
) {
    for (mut transform, speed) in &mut query {
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

        if forward_move != 0.0 || strafe_move != 0.0 {
            let forward = transform.forward();
            let right = transform.right();
            
            let mut direction = (forward * forward_move) + (right * strafe_move);
            direction.y = 0.0;
            
            if direction != Vec3::ZERO {
                direction = direction.normalize();
                transform.translation += direction * speed.0 * time.delta_seconds();
                info!("Player position: {:?}", transform.translation);
            }
        }
    }
}
