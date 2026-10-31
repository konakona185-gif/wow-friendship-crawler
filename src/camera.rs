
use bevy::input::mouse::{
    AccumulatedMouseMotion,
    AccumulatedMouseScroll,
    MouseScrollUnit,
};
use bevy::prelude::*;

use benilla_world::collision::WorldCollision;
use benilla_world::schedule::WorldStage;
use benilla_world::view::WorldCamera;

use crate::player::{CharacterInputSet, LocalPlayer};

const CAMERA_COLLISION_MARGIN: f32 = 0.05;
const CAMERA_COLLISION_RETURN_RATE: f32 = 6.0;
const CAMERA_LOOK_HEIGHT: f32 = 1.0;
const CAMERA_LOOK_SENSITIVITY: f32 = 0.003;
const CAMERA_PITCH_LIMIT: f32 = 89.0 * std::f32::consts::PI / 180.0;
const CAMERA_ZOOM_SPEED: f32 = 8.33;
const CAMERA_MIN_DISTANCE: f32 = 2.0;
const CAMERA_MAX_DISTANCE: f32 = 50.0;

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
    distance: f32,
    target_distance: f32,
    collision_distance: f32,
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
    mouse_scroll: Res<AccumulatedMouseScroll>,
    time: Res<Time>,
    world: Option<WorldCollision<'_, '_>>,
    mut orbit: Local<CameraOrbitState>,
) {
    if let Ok(player_transform) = player_query.single() {
        if let Ok((camera_entity, mut camera_transform, camera)) =
            camera_query.single_mut()
        {
            let vertical_offset = camera.height - CAMERA_LOOK_HEIGHT;
            let initial_distance = camera.distance.hypot(vertical_offset);

            if orbit.camera != Some(camera_entity) {
                orbit.camera = Some(camera_entity);
                orbit.yaw_offset = 0.0;
                orbit.pitch = -vertical_offset.atan2(camera.distance);
                orbit.distance = initial_distance;
                orbit.target_distance = initial_distance;
                orbit.collision_distance = initial_distance;
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

            let scroll_steps = match mouse_scroll.unit {
                MouseScrollUnit::Line => mouse_scroll.delta.y,
                MouseScrollUnit::Pixel => {
                    mouse_scroll.delta.y
                        / MouseScrollUnit::SCROLL_UNIT_CONVERSION_FACTOR
                }
            };

            if scroll_steps.is_finite() {
                orbit.target_distance = (orbit.target_distance - scroll_steps)
                    .clamp(CAMERA_MIN_DISTANCE, CAMERA_MAX_DISTANCE);
            }

            let delta = time.delta_secs().max(0.0);
            orbit.distance = advance_zoom_distance(
                orbit.distance,
                orbit.target_distance,
                delta,
            );

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

            let camera_forward = camera_rotation * Vec3::Z;
            let boom = camera_forward * orbit.distance;
            let mut open_distance = orbit.distance;

            if let Some(world) = world.as_ref() {
                if let Some(hit_distance) =
                    world.cast_camera(look_at_target, boom, false)
                {
                    open_distance = (hit_distance - CAMERA_COLLISION_MARGIN)
                        .clamp(0.0, orbit.distance);
                }
            }

            orbit.collision_distance = advance_collision_distance(
                orbit.collision_distance,
                open_distance,
                delta,
            );

            orbit.collision_distance =
                orbit.collision_distance.min(orbit.distance);

            camera_transform.translation =
                look_at_target + camera_forward * orbit.collision_distance;

            if orbit.collision_distance > f32::EPSILON {
                camera_transform.look_at(look_at_target, Vec3::Y);
            }
        }
    }
}

fn advance_zoom_distance(current: f32, target: f32, delta: f32) -> f32 {
    let max_step = CAMERA_ZOOM_SPEED * delta;
    current + (target - current).clamp(-max_step, max_step)
}

fn advance_collision_distance(current: f32, open: f32, delta: f32) -> f32 {
    if open < current {
        open
    } else {
        let interpolation =
            1.0 - (-CAMERA_COLLISION_RETURN_RATE * delta).exp();

        current + (open - current) * interpolation
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
}
