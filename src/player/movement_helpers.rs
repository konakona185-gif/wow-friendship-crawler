pub(super) use crate::movement::{
    descent_step, fall_step, foot_cone_ride, steep_contact_shear, try_step_up,
    vertical_probe_distance, walkable_ride_velocity, StepUp,
    CAPSULE_HEIGHT as PLAYER_CAPSULE_HEIGHT, CAPSULE_RADIUS as PLAYER_CAPSULE_RADIUS,
    FOOT_CONE_HEIGHT, GRAVITY, GROUND_NORMAL_THRESHOLD, GROUND_PROBE_DISTANCE, GROUND_PROBE_LIFT,
    LAND_PROBE_DISTANCE, OVERHANG_EPS, SKIN_WIDTH as PLAYER_SKIN_WIDTH, STEP_SLOPE_RATIO,
    STEP_SNAP_SLACK, STEP_UP_ADVANCE, STEP_UP_HEIGHT, TERMINAL_FALL_SPEED,
};

pub(super) const JUMP_SPEED: f32 = 9.0;
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn falling_uses_a_shorter_ground_probe_than_standing() {
        assert_eq!(vertical_probe_distance(0.0), GROUND_PROBE_DISTANCE);
        assert_eq!(vertical_probe_distance(1.0), GROUND_PROBE_DISTANCE);
        assert_eq!(vertical_probe_distance(-0.01), LAND_PROBE_DISTANCE);
        assert!(LAND_PROBE_DISTANCE < GROUND_PROBE_DISTANCE);
    }

    #[test]
    fn descent_is_limited_to_the_foot_cone_reach() {
        let horizontal_distance = 0.1;
        let cone_reach = horizontal_distance * STEP_SLOPE_RATIO + STEP_SNAP_SLACK;

        let (descent, support_continues) = descent_step(cone_reach * 2.0, horizontal_distance);

        assert!((descent - cone_reach).abs() < 1.0e-6);
        assert!(support_continues);

        let (full_descent, support_ends) = descent_step(cone_reach * 0.5, horizontal_distance);

        assert!((full_descent - cone_reach * 0.5).abs() < 1.0e-6);
        assert!(!support_ends);
    }

    #[test]
    fn falling_uses_average_velocity_and_respects_terminal_speed() {
        let delta = 1.0 / 60.0;
        let (end_velocity, average_velocity) = fall_step(0.0, delta);

        assert!((end_velocity - GRAVITY * delta).abs() < 1.0e-6);
        assert!((average_velocity - 0.5 * GRAVITY * delta).abs() < 1.0e-6);

        let (terminal_end, terminal_average) = fall_step(-TERMINAL_FALL_SPEED, delta);

        assert_eq!(terminal_end, -TERMINAL_FALL_SPEED);
        assert_eq!(terminal_average, -TERMINAL_FALL_SPEED);
    }
}
