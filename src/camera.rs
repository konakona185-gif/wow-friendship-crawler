
use bevy::prelude::*;

use benilla_world::collision::WorldCollision;
use benilla_world::schedule::WorldStage;

use crate::player::LocalPlayer;

const CAMERA_COLLISION_MARGIN: f32 = 0.05;

#[derive(Component)]
pub struct ThirdPersonCamera {
    pub distance: f32,
    pub height: f32,
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            camera_follow_system.in_set(WorldStage::Present),
        );
    }
}

fn camera_follow_system(
    player_query: Query<&Transform, (With<LocalPlayer>, Without<ThirdPersonCamera>)>,
    mut camera_query: Query<(&mut Transform, &ThirdPersonCamera), With<Camera>>,
    time: Res<Time>,
    world: Option<WorldCollision<'_, '_>>,
) {
    if let Ok(player_transform) = player_query.single() {
        if let Ok((mut camera_transform, camera)) = camera_query.single_mut() {
            let forward = player_transform.forward();

            let mut target_position =
                player_transform.translation
                - (forward * camera.distance)
                + (Vec3::Y * camera.height);

            let look_at_target =
                player_transform.translation + Vec3::new(0.0, 1.0, 0.0);

            let mut obstructed = false;

            if let Some(world) = world.as_ref() {
                let boom = target_position - look_at_target;
                let boom_length = boom.length();

                if boom_length > f32::EPSILON {
                    if let Some(hit_distance) =
                        world.cast_camera(look_at_target, boom, false)
                    {
                        let safe_distance = (hit_distance
                            - CAMERA_COLLISION_MARGIN)
                            .clamp(0.0, boom_length);

                        target_position =
                            look_at_target + boom * (safe_distance / boom_length);

                        obstructed = true;
                    }
                }
            }

            let smoothness = 10.0;

            if obstructed {
                camera_transform.translation = target_position;
            } else {
                camera_transform.translation = camera_transform.translation.lerp(
                    target_position,
                    smoothness * time.delta_secs(),
                );
            }

            camera_transform.look_at(look_at_target, Vec3::Y);
        }
    }
}
