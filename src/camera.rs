use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;
use crate::player::Character;

#[derive(Component)]
pub struct ThirdPersonCamera {
    pub distance: f32,
    pub height: f32,
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, camera_follow_system);
    }
}

fn camera_follow_system(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    player_query: Query<&Transform, (With<Character>, Without<ThirdPersonCamera>)>,
    mut camera_query: Query<(&mut Transform, &ThirdPersonCamera), With<Camera>>,
    time: Res<Time>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        if let Ok((mut camera_transform, camera)) = camera_query.get_single_mut() {
            let forward = player_transform.forward();
            let target_position = player_transform.translation - (forward * camera.distance) + (Vec3::Y * camera.height);
            let smoothness = 10.0;
            
            camera_transform.translation = camera_transform.translation.lerp(
                target_position,
                smoothness * time.delta_seconds()
            );
            
            let look_at_target = player_transform.translation + Vec3::new(0.0, 1.0, 0.0);
            camera_transform.look_at(look_at_target, Vec3::Y);
        }
    }
}
