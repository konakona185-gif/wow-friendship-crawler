pub(super) use crate::movement::{
    descent_step as enemy_descent_step, fall_step, foot_cone_ride as enemy_foot_cone_ride,
    steep_contact_shear as enemy_steep_contact_shear, try_step_up as try_enemy_step_up,
    vertical_probe_distance, walkable_ride_velocity as enemy_walkable_ride_velocity,
    StepUp as EnemyStepUp, CAPSULE_HEIGHT as ENEMY_CAPSULE_HEIGHT,
    CAPSULE_RADIUS as ENEMY_CAPSULE_RADIUS, FOOT_CONE_HEIGHT as ENEMY_FOOT_CONE_HEIGHT,
    GRAVITY as ENEMY_GRAVITY, GROUND_NORMAL_THRESHOLD as ENEMY_GROUND_NORMAL_THRESHOLD,
    GROUND_PROBE_DISTANCE as ENEMY_GROUND_PROBE_DISTANCE,
    GROUND_PROBE_LIFT as ENEMY_GROUND_PROBE_LIFT, LAND_PROBE_DISTANCE as ENEMY_LAND_PROBE_DISTANCE,
    OVERHANG_EPS as ENEMY_OVERHANG_EPS, SKIN_WIDTH as ENEMY_SKIN_WIDTH,
    STEP_SLOPE_RATIO as ENEMY_STEP_SLOPE_RATIO, STEP_SNAP_SLACK as ENEMY_STEP_SNAP_SLACK,
    STEP_UP_ADVANCE as ENEMY_STEP_UP_ADVANCE, STEP_UP_HEIGHT as ENEMY_STEP_UP_HEIGHT,
    TERMINAL_FALL_SPEED as ENEMY_TERMINAL_FALL_SPEED,
};
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn falling_uses_a_shorter_ground_probe_than_standing() {
        assert_eq!(vertical_probe_distance(0.0), ENEMY_GROUND_PROBE_DISTANCE);
        assert_eq!(vertical_probe_distance(1.0), ENEMY_GROUND_PROBE_DISTANCE);
        assert_eq!(vertical_probe_distance(-0.01), ENEMY_LAND_PROBE_DISTANCE);
        assert!(ENEMY_LAND_PROBE_DISTANCE < ENEMY_GROUND_PROBE_DISTANCE);
    }

    #[test]
    fn descent_is_limited_to_the_foot_cone_reach() {
        let horizontal_distance = 0.1;
        let cone_reach = horizontal_distance * ENEMY_STEP_SLOPE_RATIO + ENEMY_STEP_SNAP_SLACK;

        let (descent, support_continues) =
            enemy_descent_step(cone_reach * 2.0, horizontal_distance);

        assert!((descent - cone_reach).abs() < 1.0e-6);
        assert!(support_continues);

        let (full_descent, support_ends) =
            enemy_descent_step(cone_reach * 0.5, horizontal_distance);

        assert!((full_descent - cone_reach * 0.5).abs() < 1.0e-6);
        assert!(!support_ends);
    }

    #[test]
    fn falling_uses_average_velocity_and_respects_terminal_speed() {
        let delta = 1.0 / 60.0;
        let (end_velocity, average_velocity) = fall_step(0.0, delta);

        assert!((end_velocity - ENEMY_GRAVITY * delta).abs() < 1.0e-6);
        assert!((average_velocity - 0.5 * ENEMY_GRAVITY * delta).abs() < 1.0e-6);

        let (terminal_end, terminal_average) = fall_step(-ENEMY_TERMINAL_FALL_SPEED, delta);

        assert_eq!(terminal_end, -ENEMY_TERMINAL_FALL_SPEED);
        assert_eq!(terminal_average, -ENEMY_TERMINAL_FALL_SPEED);
    }
}
