use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;

#[derive(Component)]
pub struct Character;

#[derive(Component)]
pub struct LocalPlayer;

#[derive(Component, Default)]
pub struct CharacterCommand {
    pub movement: Vec2,
    pub jump: bool,
    pub look_delta: f32,
    pub select_target: bool,
    pub ability_1: bool,
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CharacterInputSet {
    Collect,
    Process,
}

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

#[derive(Component)]
pub struct Velocity(pub Vec3);

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (CharacterInputSet::Collect, CharacterInputSet::Process).chain(),
        )
        .add_systems(
            Update,
            local_player_command_system.in_set(CharacterInputSet::Collect),
        )
        .add_systems(
            Update,
            player_movement_system.in_set(CharacterInputSet::Process),
        );
    }
}

fn local_player_command_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut query: Query<&mut CharacterCommand, With<LocalPlayer>>,
) {
    let mut forward_move = 0.0;
    let mut strafe_move = 0.0;

    if keyboard_input.pressed(KeyCode::KeyW)
        || keyboard_input.pressed(KeyCode::ArrowUp)
    {
        forward_move += 1.0;
    }

    if keyboard_input.pressed(KeyCode::KeyS)
        || keyboard_input.pressed(KeyCode::ArrowDown)
    {
        forward_move -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::KeyA)
        || keyboard_input.pressed(KeyCode::ArrowLeft)
    {
        strafe_move -= 1.0;
    }

    if keyboard_input.pressed(KeyCode::KeyD)
        || keyboard_input.pressed(KeyCode::ArrowRight)
    {
        strafe_move += 1.0;
    }

    let mut mouse_delta = 0.0;

    if mouse_button.pressed(MouseButton::Right) {
        for event in mouse_motion.read() {
            mouse_delta += event.delta.x;
        }
    } else {
        mouse_motion.clear();
    }

    for mut command in &mut query {
        command.movement = Vec2::new(strafe_move, forward_move);
        command.jump = keyboard_input.just_pressed(KeyCode::Space);
        command.look_delta = mouse_delta;
        command.select_target = keyboard_input.just_pressed(KeyCode::Tab);
        command.ability_1 = keyboard_input.just_pressed(KeyCode::Digit1);
    }
}

fn player_movement_system(
    time: Res<Time>,
    mut query: Query<
        (
            &mut Transform,
            &MovementSpeed,
            &mut Velocity,
            &mut CharacterCommand,
        ),
        With<Character>,
    >,
) {
    let gravity = -25.0;
    let jump_force = 9.0;
    let floor_y = 1.0;

    for (mut transform, speed, mut velocity, mut command) in &mut query {
        if command.look_delta != 0.0 {
            let sensitivity = 0.003;
            transform.rotate_y(-command.look_delta * sensitivity);
        }

        command.look_delta = 0.0;

        let movement_input = command.movement;
        let jump_requested = command.jump;

        command.jump = false;

        let mut move_direction = Vec3::ZERO;

        if movement_input != Vec2::ZERO {
            let forward = transform.forward().to_array();
            let right = transform.right().to_array();

            let forward_vec = Vec3::from_array(forward);
            let right_vec = Vec3::from_array(right);

            move_direction =
                (forward_vec * movement_input.y)
                + (right_vec * movement_input.x);

            move_direction.y = 0.0;

            if move_direction != Vec3::ZERO {
                move_direction = move_direction.normalize();
            }
        }

        let is_grounded = transform.translation.y <= floor_y;

        if is_grounded {
            velocity.0.y = 0.0;
            transform.translation.y = floor_y;

            if jump_requested {
                velocity.0.y = jump_force;
            }
        } else {
            velocity.0.y += gravity * time.delta_seconds();
        }

        let horizontal_velocity = move_direction * speed.0;

        transform.translation += horizontal_velocity * time.delta_seconds();
        transform.translation.y += velocity.0.y * time.delta_seconds();

        if transform.translation.y < floor_y {
            transform.translation.y = floor_y;
            velocity.0.y = 0.0;
        }
    }
}
