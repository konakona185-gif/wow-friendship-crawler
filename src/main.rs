use bevy::prelude::*;

// =========================================================================
// 1. COMPONENTI (Dati puri)
// =========================================================================

#[derive(Component, Debug)]
enum FellowshipRole {
    Tank,
    Healer,
    Dps,
}

#[derive(Component)]
struct Character {
    name: String,
    role: FellowshipRole,
}

#[derive(Component)]
struct MovementSpeed(f32);

#[derive(Component)]
struct Health {
    current: u32,
    max: u32,
}

// =========================================================================
// 2. SISTEMI (Logica di Gioco)
// =========================================================================

// Sistema di setup: spawna la telecamera e un eroe di prova (Tank)
fn setup_game(mut commands: Commands) {
    // Spawna una telecamera 3D di base per la terza persona
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // Spawna l'entità del giocatore principale (es. un Guerriero Tank di WoW)
    commands.spawn((
        Character {
            name: "Tauren Warrior".to_string(),
            role: FellowshipRole::Tank,
        },
        Health { current: 500, max: 500 },
        MovementSpeed(5.0),
        TransformBundle::from(Transform::from_xyz(0.0, 0.0, 0.0)),
        VisibilityBundle::default(),
    ));

    println!("Eroe inizializzato correttamente! Premi W, A, S, D per muoverti in 3D.");
}

// Sistema di movimento 3D d'azione lineare (stile Fellowship / WoW libero)
fn player_movement_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&mut Transform, &MovementSpeed, &Character)>,
) {
    for (mut transform, speed, character) in query.iter_mut() {
        let mut direction = Vec3::ZERO;

        if keyboard_input.pressed(KeyCode::KeyW) {
            direction.z -= 1.0; // Avanti
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            direction.z += 1.0; // Indietro
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            direction.x -= 1.0; // Sinistra
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            direction.x += 1.0; // Destra
        }

        // Se c'è movimento, normalizza il vettore e aggiorna la posizione nello spazio 3D
        if direction != Vec3::ZERO {
            direction = direction.normalize();
            transform.translation += direction * speed.0 * time.delta_seconds();
            println!(
                "[{}] Si muove a: X: {:.2}, Z: {:.2}",
                character.name, transform.translation.x, transform.translation.z
            );
        }
    }
}

// =========================================================================
// 3. APPLICAZIONE PRINCIPALE
// =========================================================================
fn main() {
    App::new()
        // DefaultPlugins carica la finestra grafica e la logica core di Bevy
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Fellowship WoW Crawler".into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup_game)
        .add_systems(Update, player_movement_system)
        .run();
}
