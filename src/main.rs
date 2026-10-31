mod player;
mod camera;
mod combat;

use bevy::prelude::*;
use player::{Character, FellowshipRole, MovementSpeed, Velocity, PlayerPlugin};
use camera::{ThirdPersonCamera, CameraPlugin};
use combat::{Target, Enemy, CombatStats, CombatPlugin};

fn main() {
    let mut app = App::new();

    if std::env::var("CI").is_ok() {
        app.add_plugins(MinimalPlugins);
    } else {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Fellowship Fan Project (Bevy 0.14)".into(),
                ..default()
            }),
            ..default()
        }));
    }

    app.add_plugins((PlayerPlugin, CameraPlugin, CombatPlugin))
       .add_systems(Startup, setup_game)
       .run();
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
        transform: Transform::from_xyz(10.0, 20.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
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
        CombatStats { current_health: 100.0, max_health: 100.0, resource: 0.0 },
        PbrBundle {
            mesh: meshes.add(Capsule3d::new(0.5, 1.0)),
            material: materials.add(Color::from(LinearRgba::new(0.0, 0.8, 0.2, 1.0))),
            transform: Transform::from_xyz(0.0, 1.0, 0.0),
            ..default()
        },
    ));

    commands.spawn((
        Enemy,
        CombatStats { current_health: 150.0, max_health: 150.0, resource: 0.0 },
        PbrBundle {
            mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            material: materials.add(Color::from(LinearRgba::new(0.9, 0.1, 0.1, 1.0))),
            transform: Transform::from_xyz(5.0, 0.5, -5.0),
            ..default()
        },
    ));

    commands.spawn((
        ThirdPersonCamera { distance: 8.0, height: 4.5 },
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 4.5, 8.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
            ..default()
        },
    ));
}
