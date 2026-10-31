
use bevy::prelude::*;

use benilla_world::collision::WorldCollision;
use benilla_world::schedule::WorldStage;
use benilla_world::view::WorldCamera;

use crate::player::{CharacterInputSet, LocalPlayer};

const CAMERA_COLLISION_MARGIN: f32 = 0.05;
const CAMERA_SMOOTHNESS: f32 = 10.0;
const CAMERA_LOOK_HEIGHT: f32 = 1.0;

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
            camera_follow_system
                .in_set(WorldStage::Input)
                .after(CharacterInputSet::Process),
        );
    }
}

fn camera_follow_system(
    player_query: Query<&Transform, (With<LocalPlayer>, Without<ThirdPersonCamera>)>,
    mut camera_query: Query<(&mut Transform, &ThirdPersonCamera), With<WorldCamera>>,
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
                player_transform.translation
                + Vec3::Y * CAMERA_LOOK_HEIGHT;

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

            let next_position = if obstructed {
                target_position
            } else {
                let delta = time.delta_secs();
                let interpolation =
                    1.0 - (-CAMERA_SMOOTHNESS * delta).exp();

                camera_transform.translation.lerp(
                    target_position,
                    interpolation,
                )
            };

            let movement = next_position - camera_transform.translation;
            let movement_length = movement.length();

            if movement_length > f32::EPSILON {
                if let Some(world) = world.as_ref() {
                    if let Some(hit_distance) = world.cast_camera(
                        camera_transform.translation,
                        movement,
                        false,
                    ) {
                        let safe_distance = (hit_distance
                            - CAMERA_COLLISION_MARGIN)
                            .clamp(0.0, movement_length);

                        camera_transform.translation +=
                            movement * (safe_distance / movement_length);
                    } else {
                        camera_transform.translation = next_position;
                    }
                } else {
                    camera_transform.translation = next_position;
                }
            }

            if camera_transform.translation.distance_squared(look_at_target)
                > f32::EPSILON
            {
                camera_transform.look_at(look_at_target, Vec3::Y);
            }
        }
    }
}
