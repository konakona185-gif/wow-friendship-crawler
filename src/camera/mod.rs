use bevy::color::Alpha;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;
use bevy::render::alpha::AlphaMode;

use benilla_world::collision::WorldCollision;
use benilla_world::model_fade::{self_model_fade_alpha, SELF_FADE_WINDOW};
use benilla_world::schedule::WorldStage;
use benilla_world::view::{ViewDistance, Viewer, WorldCamera};

use crate::player::{CharacterInputSet, LocalPlayer};

const CAMERA_COLLISION_MARGIN: f32 = 0.05;
const CAMERA_COLLISION_RETURN_RATE: f32 = 6.0;
const CAMERA_LOOK_HEIGHT: f32 = 1.0;
const CAMERA_LOOK_SENSITIVITY: f32 = 0.003;
const CAMERA_PITCH_LIMIT: f32 = 89.0 * std::f32::consts::PI / 180.0;
const CAMERA_ZOOM_SPEED: f32 = 8.33;
const CAMERA_MIN_DISTANCE: f32 = 0.0;
const CAMERA_MAX_DISTANCE: f32 = 50.0;
const CAMERA_FOLLOW_SPEED: f32 = std::f32::consts::PI;
const CAMERA_FOLLOW_MIN_TIME: f32 = 0.1;
const CAMERA_FOLLOW_MAX_TIME: f32 = 2.0;
const CAMERA_FOLLOW_EPSILON: f32 = 0.001;

const CAMERA_CMD_RIGHT_MOUSE: u32 = 0x1;
const CAMERA_CMD_LEFT_MOUSE: u32 = 0x2;
const CAMERA_CMD_FORWARD: u32 = 0x10;
const CAMERA_CMD_BACKWARD: u32 = 0x20;
const CAMERA_CMD_STRAFE_LEFT: u32 = 0x40;
const CAMERA_CMD_STRAFE_RIGHT: u32 = 0x80;

const CAMERA_CMD_MOVE_BITS: u32 = CAMERA_CMD_FORWARD | CAMERA_CMD_BACKWARD;
const CAMERA_CMD_STRAFE_BITS: u32 = CAMERA_CMD_STRAFE_LEFT | CAMERA_CMD_STRAFE_RIGHT;
const CAMERA_CMD_MOVEMENT_BITS: u32 = CAMERA_CMD_MOVE_BITS | CAMERA_CMD_STRAFE_BITS;
const CAMERA_CMD_ARM_BITS: u32 =
    CAMERA_CMD_MOVEMENT_BITS | CAMERA_CMD_RIGHT_MOUSE | CAMERA_CMD_LEFT_MOUSE;

#[derive(Component)]
pub struct ThirdPersonCamera {
    pub distance: f32,
    pub height: f32,
}

#[derive(Clone, Copy)]
struct CameraFollowReturn {
    from: f32,
    elapsed: f32,
    duration: f32,
}

#[derive(Default)]
struct CameraOrbitState {
    camera: Option<Entity>,
    yaw_offset: f32,
    pitch: f32,
    distance: f32,
    target_distance: f32,
    collision_distance: f32,
    last_command: Option<u32>,
    follow_return: Option<CameraFollowReturn>,
    player_material: Option<Handle<StandardMaterial>>,
    material_base_color: Option<Color>,
    material_alpha_mode: Option<AlphaMode>,
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
    player_query: Query<
        (&Transform, Option<&MeshMaterial3d<StandardMaterial>>),
        (With<LocalPlayer>, Without<ThirdPersonCamera>),
    >,
    mut camera_query: Query<(Entity, &mut Transform, &ThirdPersonCamera), With<WorldCamera>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    time: Res<Time>,
    world: Option<WorldCollision<'_, '_>>,
    view_distance: Option<Res<ViewDistance>>,
    mut viewer: Option<ResMut<Viewer>>,
    material_assets: Option<ResMut<Assets<StandardMaterial>>>,
    mut orbit: Local<CameraOrbitState>,
) {
    if let Ok((player_transform, player_material)) = player_query.single() {
        if let Ok((camera_entity, mut camera_transform, camera)) = camera_query.single_mut() {
            let vertical_offset = camera.height - CAMERA_LOOK_HEIGHT;
            let initial_distance = camera.distance.hypot(vertical_offset);

            if orbit.camera != Some(camera_entity) {
                orbit.camera = Some(camera_entity);
                orbit.yaw_offset = 0.0;
                orbit.pitch = -vertical_offset.atan2(camera.distance);
                orbit.distance = initial_distance;
                orbit.target_distance = initial_distance;
                orbit.collision_distance = initial_distance;
                orbit.last_command = None;
                orbit.follow_return = None;
            }

            let right_held = mouse_button.pressed(MouseButton::Right);
            let left_held = mouse_button.pressed(MouseButton::Left);
            let look_held = right_held || left_held;

            if look_held {
                if left_held && !right_held {
                    orbit.yaw_offset -= mouse_motion.delta.x * CAMERA_LOOK_SENSITIVITY;
                }

                orbit.pitch = (orbit.pitch - mouse_motion.delta.y * CAMERA_LOOK_SENSITIVITY)
                    .clamp(-CAMERA_PITCH_LIMIT, CAMERA_PITCH_LIMIT);
            }

            orbit.yaw_offset = wrap_camera_angle(orbit.yaw_offset);

            let command_word = camera_command_word(&keyboard_input, &mouse_button);

            let delta = time.delta_secs().max(0.0);

            update_camera_follow(&mut orbit, command_word, look_held, delta);

            let scroll_steps = match mouse_scroll.unit {
                MouseScrollUnit::Line => mouse_scroll.delta.y,
                MouseScrollUnit::Pixel => {
                    mouse_scroll.delta.y / MouseScrollUnit::SCROLL_UNIT_CONVERSION_FACTOR
                }
            };

            if scroll_steps.is_finite() {
                orbit.target_distance = (orbit.target_distance - scroll_steps)
                    .clamp(CAMERA_MIN_DISTANCE, CAMERA_MAX_DISTANCE);
            }

            orbit.distance = advance_zoom_distance(orbit.distance, orbit.target_distance, delta);

            let player_yaw = player_transform.rotation.to_euler(EulerRot::YXZ).0;

            let camera_rotation = Quat::from_euler(
                EulerRot::YXZ,
                player_yaw + orbit.yaw_offset,
                orbit.pitch,
                0.0,
            );

            let look_at_target = player_transform.translation + Vec3::Y * CAMERA_LOOK_HEIGHT;

            let camera_forward = camera_rotation * Vec3::Z;
            let boom = camera_forward * orbit.distance;
            let mut open_distance = orbit.distance;

            if orbit.distance > f32::EPSILON {
                if let Some(world) = world.as_ref() {
                    if let Some(hit_distance) = world.cast_camera(look_at_target, boom, false) {
                        open_distance =
                            (hit_distance - CAMERA_COLLISION_MARGIN).clamp(0.0, orbit.distance);
                    }
                }
            }

            orbit.collision_distance =
                advance_collision_distance(orbit.collision_distance, open_distance, delta);

            orbit.collision_distance = orbit.collision_distance.min(orbit.distance);

            camera_transform.translation =
                look_at_target + camera_forward * orbit.collision_distance;
            camera_transform.rotation = camera_rotation;

            let nearclip = view_distance
                .as_deref()
                .map_or(0.1, |distance| distance.nearclip);

            let self_fade =
                self_model_fade_alpha(orbit.collision_distance, nearclip, SELF_FADE_WINDOW);

            if let Some(viewer) = viewer.as_deref_mut() {
                viewer.self_fade = self_fade;
            }

            update_player_material_fade(player_material, material_assets, &mut orbit, self_fade);
        }
    }
}

fn update_player_material_fade(
    player_material: Option<&MeshMaterial3d<StandardMaterial>>,
    mut materials: Option<ResMut<Assets<StandardMaterial>>>,
    orbit: &mut CameraOrbitState,
    alpha: f32,
) {
    let Some(player_material) = player_material else {
        return;
    };

    let handle = player_material.0.clone();

    if orbit.player_material.as_ref() != Some(&handle) {
        let Some(assets) = materials.as_deref() else {
            return;
        };

        let Some(material) = assets.get(&handle) else {
            return;
        };

        orbit.player_material = Some(handle.clone());
        orbit.material_base_color = Some(material.base_color);
        orbit.material_alpha_mode = Some(material.alpha_mode);
    }

    let Some(assets) = materials.as_deref_mut() else {
        return;
    };

    let Some(material) = assets.get_mut(&handle) else {
        return;
    };

    let Some(base_color) = orbit.material_base_color else {
        return;
    };

    let alpha = alpha.clamp(0.0, 1.0);
    let target_color = base_color.with_alpha(base_color.alpha() * alpha);
    let original_mode = orbit.material_alpha_mode.unwrap_or(AlphaMode::Opaque);

    let target_mode = if alpha < 0.9999 {
        AlphaMode::Blend
    } else {
        original_mode
    };

    if material.base_color != target_color {
        material.base_color = target_color;
    }

    if material.alpha_mode != target_mode {
        material.alpha_mode = target_mode;
    }
}

fn camera_command_word(keyboard: &ButtonInput<KeyCode>, mouse: &ButtonInput<MouseButton>) -> u32 {
    let mut word = 0;

    if mouse.pressed(MouseButton::Right) {
        word |= CAMERA_CMD_RIGHT_MOUSE;
    }

    if mouse.pressed(MouseButton::Left) {
        word |= CAMERA_CMD_LEFT_MOUSE;
    }

    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        word |= CAMERA_CMD_FORWARD;
    }

    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        word |= CAMERA_CMD_BACKWARD;
    }

    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        word |= CAMERA_CMD_STRAFE_LEFT;
    }

    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        word |= CAMERA_CMD_STRAFE_RIGHT;
    }

    word
}

fn update_camera_follow(orbit: &mut CameraOrbitState, command: u32, look_held: bool, delta: f32) {
    let previous = orbit.last_command.replace(command);

    if look_held {
        orbit.follow_return = None;
        return;
    }

    if let Some(previous) = previous {
        let changed = (previous ^ command) & CAMERA_CMD_ARM_BITS != 0;

        if changed {
            let stopping = (previous & !command) & CAMERA_CMD_MOVEMENT_BITS != 0;

            let moving = command & CAMERA_CMD_MOVEMENT_BITS != 0;

            if moving && !stopping {
                if orbit.follow_return.is_none() {
                    orbit.follow_return = start_camera_follow(orbit.yaw_offset);
                }
            } else {
                orbit.follow_return = None;
            }
        }
    }

    advance_camera_follow(&mut orbit.follow_return, &mut orbit.yaw_offset, delta);
}

fn start_camera_follow(yaw_offset: f32) -> Option<CameraFollowReturn> {
    let from = wrap_camera_angle(yaw_offset);

    if from.abs() < CAMERA_FOLLOW_EPSILON {
        return None;
    }

    let duration =
        (from.abs() / CAMERA_FOLLOW_SPEED).clamp(CAMERA_FOLLOW_MIN_TIME, CAMERA_FOLLOW_MAX_TIME);

    Some(CameraFollowReturn {
        from,
        elapsed: 0.0,
        duration,
    })
}

fn advance_camera_follow(
    follow: &mut Option<CameraFollowReturn>,
    yaw_offset: &mut f32,
    delta: f32,
) {
    let Some(arm) = follow.as_mut() else {
        return;
    };

    arm.elapsed += delta;

    let progress = (arm.elapsed / arm.duration).clamp(0.0, 1.0);
    let easing = (1.0 - (std::f32::consts::PI * progress).cos()) * 0.5;

    *yaw_offset = wrap_camera_angle(arm.from * (1.0 - easing));

    if progress >= 1.0 {
        *yaw_offset = 0.0;
        *follow = None;
    }
}

fn wrap_camera_angle(angle: f32) -> f32 {
    (angle + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI
}

fn advance_zoom_distance(current: f32, target: f32, delta: f32) -> f32 {
    let max_step = CAMERA_ZOOM_SPEED * delta;
    current + (target - current).clamp(-max_step, max_step)
}

fn advance_collision_distance(current: f32, open: f32, delta: f32) -> f32 {
    if open < current {
        open
    } else {
        let interpolation = 1.0 - (-CAMERA_COLLISION_RETURN_RATE * delta).exp();

        current + (open - current) * interpolation
    }
}

#[cfg(test)]
mod tests;
