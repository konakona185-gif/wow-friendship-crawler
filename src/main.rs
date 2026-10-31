
mod player;
mod camera;
mod combat;

use bevy::prelude::*;
use bevy::input::InputPlugin;
use player::{Character, FellowshipRole, MovementSpeed, Velocity, PlayerPlugin};
use camera::{ThirdPersonCamera, CameraPlugin};
use combat::{Target, Enemy, CombatStats, CombatPlugin};

fn main() {
    let mut app = App::new();

    let is_headless = std::env::var("CI").is_ok();

    if is_headless {
        app.add_plugins(MinimalPlugins)
            .add_plugins(InputPlugin)
            .add_systems(Startup, setup_headless_game);
    } else {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Fellowship Fan Project (Bevy 0.14)".into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup_game);
    }

    app.add_plugins((PlayerPlugin, CameraPlugin, CombatPlugin))
        .run();
}

fn setup_headless_game(mut commands: Commands) {
    commands.spawn((
        Character,
        FellowshipRole::Tank,
        MovementSpeed(8.0),
        Velocity(Vec3::ZERO),
        Target { entity: None },
        CombatStats {
            current_health: 100.0,
            max_health: 100.0,
            resource: 0.0,
        },
        Transform::from_xyz(0.0, 1.0, 0.0),
        GlobalTransform::default(),
    ));

    let enemy_positions = [
        Vec3::new(5.0, 0.5, -5.0),
        Vec3::new(-6.0, 0.5, -8.0),
        Vec3::new(2.0, 0.5, -12.0),
    ];

    for (i, pos) in enemy_positions.iter().enumerate() {
        let health = 50.0 + (i as f32 * 20.0);

        commands.spawn((
            Enemy,
            CombatStats {
                current_health: health,
                max_health: health,
                resource: 0.0,
            },
            Transform::from_translation(*pos),
            GlobalTransform::default(),
        ));
    }
}

fn setup_game(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            illuminance: 4000.0,
            ..default()
        },
        transform: Transform::from_xyz(10.0, 20.0, 10.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::default().mesh().size(50.0, 50.0)),
        material: materials.add(Color::from(LinearRgba::new(0.2, 0.2, 0.2, 1.0))),
        ..default()
    });

    commands.spawn((
        Character,
        FellowshipRole::Tank,
        MovementSpeed(8.0),
        Velocity(Vec3::ZERO),
        Target { entity: None },
        CombatStats {
            current_health: 100.0,
            max_health: 100.0,
            resource: 0.0,
        },
        PbrBundle {
            mesh: meshes.add(Capsule3d::new(0.5, 1.0)),
            material: materials.add(Color::from(LinearRgba::new(0.0, 0.8, 0.2, 1.0))),
            transform: Transform::from_xyz(0.0, 1.0, 0.0),
            ..default()
        },
    ));

    let enemy_positions = [
        Vec3::new(5.0, 0.5, -5.0),
        Vec3::new(-6.0, 0.5, -8.0),
        Vec3::new(2.0, 0.5, -12.0),
    ];

    for (i, pos) in enemy_positions.iter().enumerate() {
        let health = 50.0 + (i as f32 * 20.0);

        commands.spawn((
            Enemy,
            CombatStats {
                current_health: health,
                max_health: health,
                resource: 0.0,
            },
            PbrBundle {
                mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
                material: materials.add(Color::from(LinearRgba::new(0.9, 0.1, 0.1, 1.0))),
                transform: Transform::from_translation(*pos),
                ..default()
            },
        ));
    }

    commands.spawn((
        ThirdPersonCamera {
            distance: 8.0,
            height: 4.5,
        },
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 4.5, 8.0)
                .looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
            ..default()
        },
    ));
}
