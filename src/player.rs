use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;

use benilla_assets::AdtTile;
use benilla_world::terrain_stream::{
    terrain_height_under,
    TerrainStreamer,
};

#[derive(Component)]
pub struct Character;

#[derive(Component)]
pub struct LocalPlayer;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PlayerId(pub u64);

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
    mouse_motion: Res<AccumulatedMouseMotion>,
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

    let mouse_delta = if mouse_button.pressed(MouseButton::Right) {
        mouse_motion.delta.x
    } else {
        0.0
    };

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
    terrain_streamer: Option<Res<TerrainStreamer>>,
    adt_tiles: Option<Res<Assets<AdtTile>>>,
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
    let floor_offset = 1.0;
    let delta = time.delta_secs();

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

        let horizontal_velocity = move_direction * speed.0;
        transform.translation += horizontal_velocity * delta;

        let (Some(streamer), Some(tiles)) =
            (terrain_streamer.as_deref(), adt_tiles.as_deref())
        else {
            continue;
        };

        let Some(terrain_height) =
            terrain_height_under(streamer, tiles, transform.translation)
        else {
            continue;
        };

        let floor_y = terrain_height + floor_offset;

        let is_grounded =
            transform.translation.y <= floor_y + 0.05
            && velocity.0.y <= 0.0;

        if is_grounded {
            transform.translation.y = floor_y;

            if jump_requested {
                velocity.0.y = jump_force;
            } else {
                velocity.0.y = 0.0;
            }
        } else {
            velocity.0.y += gravity * delta;
        }

        transform.translation.y += velocity.0.y * delta;

        if transform.translation.y <= floor_y {
            transform.translation.y = floor_y;
            velocity.0.y = 0.0;
        }
    }
}
