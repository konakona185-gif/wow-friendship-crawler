use avian3d::prelude::Collider;
use benilla_world::collision::WorldCollision;
use bevy::prelude::*;

pub(crate) const CAPSULE_RADIUS: f32 = 1.0 / 3.0;
pub(crate) const CAPSULE_HEIGHT: f32 = 2.027_777_7;
pub(crate) const SKIN_WIDTH: f32 = 0.02;
pub(crate) const GROUND_NORMAL_THRESHOLD: f32 = 0.642_788;
pub(crate) const GROUND_PROBE_LIFT: f32 = 0.02;
pub(crate) const GROUND_PROBE_DISTANCE: f32 = 0.18;
pub(crate) const LAND_PROBE_DISTANCE: f32 = 0.03;
pub(crate) const STEP_UP_HEIGHT: f32 = 1.0;
pub(crate) const STEP_UP_ADVANCE: f32 = 1.191_753_6;
pub(crate) const STEP_SLOPE_RATIO: f32 = 1.849_399;
pub(crate) const STEP_SNAP_SLACK: f32 = 0.027_777_8;
pub(crate) const FOOT_CONE_HEIGHT: f32 = CAPSULE_RADIUS * STEP_SLOPE_RATIO;
pub(crate) const OVERHANG_EPS: f32 = -1.0e-4;
pub(crate) const GRAVITY: f32 = -25.0;
pub(crate) const TERMINAL_FALL_SPEED: f32 = 60.148_003;

#[derive(Clone, Copy)]
pub(crate) struct StepUp {
    pub(crate) rise: f32,
    pub(crate) ride_to: Option<f32>,
}

pub(crate) fn descent_step(drop: f32, horizontal_distance: f32) -> (f32, bool) {
    let cone_reach = horizontal_distance * STEP_SLOPE_RATIO + STEP_SNAP_SLACK;

    (drop.min(cone_reach), drop > cone_reach)
}

pub(crate) fn vertical_probe_distance(vertical_velocity: f32) -> f32 {
    if vertical_velocity < 0.0 {
        LAND_PROBE_DISTANCE
    } else {
        GROUND_PROBE_DISTANCE
    }
}

pub(crate) fn fall_step(initial_velocity: f32, delta: f32) -> (f32, f32) {
    let terminal_velocity = -TERMINAL_FALL_SPEED;
    let unconstrained_end = initial_velocity + GRAVITY * delta;

    if unconstrained_end >= terminal_velocity {
        return (
            unconstrained_end,
            0.5 * (initial_velocity + unconstrained_end),
        );
    }

    if initial_velocity <= terminal_velocity {
        return (terminal_velocity, terminal_velocity);
    }

    let time_to_terminal = (terminal_velocity - initial_velocity) / GRAVITY;

    let displacement = 0.5 * (initial_velocity + terminal_velocity) * time_to_terminal
        + terminal_velocity * (delta - time_to_terminal);

    (terminal_velocity, displacement / delta)
}

pub(crate) fn try_step_up(
    world: &WorldCollision<'_, '_>,
    capsule: &Collider,
    center: Vec3,
    direction: Vec3,
    travel: f32,
) -> Option<StepUp> {
    let cast = |from: Vec3, movement: Vec3| world.cast_mover(capsule, from, movement, SKIN_WIDTH);

    let ahead = cast(center, direction * travel)?;
    let normal = ahead.face_normal;

    if normal.y >= GROUND_NORMAL_THRESHOLD
        || normal.y < OVERHANG_EPS
        || normal.dot(direction) >= 0.0
    {
        return None;
    }

    let feet_y = center.y - CAPSULE_HEIGHT * 0.5;
    let contact_height = ahead.point1.y - feet_y;

    let rise = cast(center, Vec3::Y * STEP_UP_HEIGHT).map_or(STEP_UP_HEIGHT, |hit| hit.distance);

    if rise < 1.0e-3 {
        return None;
    }

    let raised = center + Vec3::Y * rise;
    let advance = travel.max(STEP_UP_ADVANCE);

    let forward_distance = cast(raised, direction * advance).map_or(advance, |hit| hit.distance);

    let over = raised + direction * forward_distance;

    let reach = rise + advance * STEP_SLOPE_RATIO + STEP_SNAP_SLACK;

    let down = cast(over, Vec3::NEG_Y * reach)?;

    if down.face_normal.y < GROUND_NORMAL_THRESHOLD {
        return None;
    }

    let landed = over + Vec3::NEG_Y * down.distance;
    let height_gain = landed.y - center.y;

    if height_gain <= 0.05 {
        return None;
    }

    let ride_to = (contact_height <= FOOT_CONE_HEIGHT).then_some(landed.y);

    Some(StepUp { rise, ride_to })
}

pub(crate) fn walkable_ride_velocity(normal: Vec3, velocity: Vec3) -> Option<Vec3> {
    if normal.y < GROUND_NORMAL_THRESHOLD || velocity.dot(normal) >= 0.0 {
        return None;
    }

    Some(Vec3::new(
        velocity.x,
        -(velocity.x * normal.x + velocity.z * normal.z) / normal.y,
        velocity.z,
    ))
}

pub(crate) fn foot_cone_ride(normal: Vec3, velocity: Vec3) -> Option<Vec3> {
    if !(OVERHANG_EPS..GROUND_NORMAL_THRESHOLD).contains(&normal.y) {
        return None;
    }

    let horizontal_normal = Vec3::new(normal.x, 0.0, normal.z).normalize_or_zero();

    let into = -(velocity.x * horizontal_normal.x + velocity.z * horizontal_normal.z);

    if into <= 0.0 {
        return None;
    }

    Some(Vec3::new(velocity.x, into * STEP_SLOPE_RATIO, velocity.z))
}

pub(crate) fn steep_contact_shear(normal: Vec3, velocity: Vec3) -> Option<Vec3> {
    if !(OVERHANG_EPS..GROUND_NORMAL_THRESHOLD).contains(&normal.y) {
        return None;
    }

    let normal_velocity = velocity.dot(normal);

    if normal_velocity >= 0.0 {
        return None;
    }

    let horizontal_normal = Vec3::new(normal.x, 0.0, normal.z);
    let horizontal_normal_length_squared = horizontal_normal.length_squared();

    if horizontal_normal_length_squared <= f32::EPSILON {
        return None;
    }

    Some(velocity - (normal_velocity / horizontal_normal_length_squared) * horizontal_normal)
}
