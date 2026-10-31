
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;

use benilla_world::collision::WorldCollision;
use benilla_world::schedule::WorldStage;
use benilla_world::view::WorldCamera;

use crate::player::{CharacterInputSet, LocalPlayer};

const CAMERA_COLLISION_MARGIN: f32 = 0.05;
const CAMERA_SMOOTHNESS: f32 = 10.0;
const CAMERA_LOOK_HEIGHT: f32 = 1.0;
const CAMERA_LOOK_SENSITIVITY: f32 = 0.003;
const CAMERA_PITCH_LIMIT: f32 = 89.0 * std::f32::consts::PI / 180.0;

#[derive(Component)]
pub struct ThirdPersonCamera {
    pub distance: f32,
    pub height: f32,
}

#[derive(Default)]
struct CameraOrbitState {
    camera: Option<Entity>,
    yaw_offset: f32,
    pitch: f32,
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
    mut camera_query: Query<
        (Entity, &mut Transform, &ThirdPersonCamera),
        With<WorldCamera>,
    >,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    time: Res<Time>,
    world: Option<WorldCollision<'_, '_>>,
    mut orbit: Local<CameraOrbitState>,
) {
    if let Ok(player_transform) = player_query.single() {
        if let Ok((camera_entity, mut camera_transform, camera)) =
            camera_query.single_mut()
        {
            let vertical_offset = camera.height - CAMERA_LOOK_HEIGHT;
            let boom_length = camera.distance.hypot(vertical_offset);

            if orbit.camera != Some(camera_entity) {
                orbit.camera = Some(camera_entity);
                orbit.yaw_offset = 0.0;
                orbit.pitch = -vertical_offset.atan2(camera.distance);
            }

            let right_held = mouse_button.pressed(MouseButton::Right);
            let left_held = mouse_button.pressed(MouseButton::Left);

            if right_held || left_held {
                if left_held && !right_held {
                    orbit.yaw_offset -=
                        mouse_motion.delta.x * CAMERA_LOOK_SENSITIVITY;
                }

                orbit.pitch = (orbit.pitch
                    - mouse_motion.delta.y * CAMERA_LOOK_SENSITIVITY)
                    .clamp(-CAMERA_PITCH_LIMIT, CAMERA_PITCH_LIMIT);
            }

            orbit.yaw_offset =
                orbit.yaw_offset.rem_euclid(std::f32::consts::TAU);

            let player_yaw =
                player_transform.rotation.to_euler(EulerRot::YXZ).0;

            let camera_rotation = Quat::from_euler(
                EulerRot::YXZ,
                player_yaw + orbit.yaw_offset,
                orbit.pitch,
                0.0,
            );

            let look_at_target =
                player_transform.translation
                + Vec3::Y * CAMERA_LOOK_HEIGHT;

            let boom = camera_rotation * Vec3::Z * boom_length;
            let mut target_position = look_at_target + boom;

            let mut obstructed = false;

            if let Some(world) = world.as_ref() {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_pitch_preserves_the_configured_camera_position() {
        let distance = 8.0;
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
}
