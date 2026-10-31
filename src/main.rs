use bevy::prelude::*;

#[derive(Component)]
pub struct Character;

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
pub struct ThirdPersonCamera {
    pub offset: Vec3,
}

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

    app.add_systems(Startup, setup_game)
       .add_systems(Update, (player_movement_system, camera_follow_system))
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
        Health { current: 100.0, max: 100.0 },
        MovementSpeed(8.0),
        PbrBundle {
            mesh: meshes.add(Capsule3d::new(0.5, 1.0)),
            material: materials.add(Color::from(LinearRgba::new(0.0, 0.8, 0.2, 1.0))),
            transform: Transform::from_xyz(0.0, 1.0, 0.0),
            ..default()
        },
    ));

    let camera_offset = Vec3::new(0.0, 4.5, 8.0);
    
    commands.spawn((
        ThirdPersonCamera { offset: camera_offset },
        Camera3dBundle {
            transform: Transform::from_translation(camera_offset)
                .looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
            ..default()
        },
    ));
}

fn player_movement_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &MovementSpeed), With<Character>>,
) {
    for (mut transform, speed) in &mut query {
        let mut direction = Vec3::ZERO;

        if keyboard_input.pressed(KeyCode::KeyW) || keyboard_input.pressed(KeyCode::ArrowUp) {
            direction.z -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyS) || keyboard_input.pressed(KeyCode::ArrowDown) {
            direction.z += 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::ArrowLeft) {
            direction.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::KeyD) || keyboard_input.pressed(KeyCode::ArrowRight) {
            direction.x += 1.0;
        }

        if direction != Vec3::ZERO {
            direction = direction.normalize();
            transform.translation += direction * speed.0 * time.delta_seconds();
            info!("Player position: {:?}", transform.translation);
        }
    }
}

fn camera_follow_system(
    player_query: Query<&Transform, (With<Character>, Without<ThirdPersonCamera>)>,
    mut camera_query: Query<(&mut Transform, &ThirdPersonCamera), With<Camera>>,
    time: Res<Time>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        if let Ok((mut camera_transform, camera)) = camera_query.get_single_mut() {
            let target_position = player_transform.translation + camera.offset;
            let smoothness = 5.0; 
            
            camera_transform.translation = camera_transform.translation.lerp(
                target_position, 
                smoothness * time.delta_seconds()
            );
            
            let look_at_target = player_transform.translation + Vec3::new(0.0, 1.0, 0.0);
            camera_transform.look_at(look_at_target, Vec3::Y);
        }
    }
}
